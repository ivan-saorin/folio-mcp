//! Depreciation functions: sln, ddb, syd, vdb, depreciation_schedule

use folio_plugin::prelude::*;
use crate::helpers::*;
use std::collections::HashMap;

// ============ SLN (Straight-Line) ============

pub struct Sln;

static SLN_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "cost",
        typ: "Number",
        description: "Initial cost of asset",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "salvage",
        typ: "Number",
        description: "Salvage value at end of life",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "life",
        typ: "Number",
        description: "Useful life in periods",
        optional: false,
        default: None,
    },
];

static SLN_EXAMPLES: [&str; 1] = ["sln(100000, 10000, 10) → 9000"];

static SLN_RELATED: [&str; 3] = ["ddb", "syd", "depreciation_schedule"];

impl FunctionPlugin for Sln {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sln",
            description: "Straight-line depreciation: (cost - salvage) / life",
            usage: "sln(cost, salvage, life)",
            args: &SLN_ARGS,
            returns: "Number",
            examples: &SLN_EXAMPLES,
            category: "finance/depreciation",
            source: None,
            related: &SLN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("sln", 3, args.len()));
        }

        let cost = match extract_number(&args[0], "sln", "cost") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let salvage = match extract_number(&args[1], "sln", "salvage") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let life = match extract_number(&args[2], "sln", "life") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if life.is_zero() {
            return Value::Error(FolioError::domain_error("sln: life must be positive"));
        }

        let depreciation = cost.sub(&salvage).checked_div(&life);
        match depreciation {
            Ok(d) => Value::Number(d),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ DDB (Double Declining Balance) ============

pub struct Ddb;

static DDB_ARGS: [ArgMeta; 5] = [
    ArgMeta {
        name: "cost",
        typ: "Number",
        description: "Initial cost of asset",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "salvage",
        typ: "Number",
        description: "Salvage value at end of life",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "life",
        typ: "Number",
        description: "Useful life in periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "period",
        typ: "Number",
        description: "Period to calculate depreciation for",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "factor",
        typ: "Number",
        description: "Depreciation factor (default: 2 for double)",
        optional: true,
        default: Some("2"),
    },
];

static DDB_EXAMPLES: [&str; 3] = [
    "ddb(100000, 10000, 10, 1) → 20000",
    "ddb(100000, 10000, 10, 2) → 16000",
    "ddb(100000, 10000, 10, 5, 2) → 8192",
];

static DDB_RELATED: [&str; 3] = ["sln", "syd", "vdb"];

impl FunctionPlugin for Ddb {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ddb",
            description: "Double declining balance depreciation",
            usage: "ddb(cost, salvage, life, period, [factor])",
            args: &DDB_ARGS,
            returns: "Number",
            examples: &DDB_EXAMPLES,
            category: "finance/depreciation",
            source: None,
            related: &DDB_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 4 {
            return Value::Error(FolioError::arg_count("ddb", 4, args.len()));
        }

        let cost = match extract_number(&args[0], "ddb", "cost") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let salvage = match extract_number(&args[1], "ddb", "salvage") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let life = match extract_number(&args[2], "ddb", "life") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let period = match extract_number(&args[3], "ddb", "period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let factor = extract_optional_number(args, 4).unwrap_or_else(|| Number::from_i64(2));

        if life.is_zero() {
            return Value::Error(FolioError::domain_error("ddb: life must be positive"));
        }

        match calculate_ddb(&cost, &salvage, &life, period, &factor) {
            Ok(d) => Value::Number(d),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_ddb(
    cost: &Number,
    salvage: &Number,
    life: &Number,
    period: i64,
    factor: &Number,
) -> Result<Number, FolioError> {
    // Rate = factor / life
    let rate = factor.checked_div(life)?;
    let one = Number::from_i64(1);

    // Calculate book value at start of period
    let mut book_value = cost.clone();

    for p in 1..period {
        let depreciation = book_value.mul(&rate);
        book_value = book_value.sub(&depreciation);

        // Don't go below salvage
        if &book_value < salvage {
            book_value = salvage.clone();
            break;
        }
    }

    // Depreciation for this period
    let depreciation = book_value.mul(&rate);

    // Don't depreciate below salvage
    let max_depreciation = book_value.sub(salvage);
    if &depreciation > &max_depreciation && !max_depreciation.is_negative() {
        Ok(max_depreciation)
    } else if max_depreciation.is_negative() {
        Ok(Number::from_i64(0))
    } else {
        Ok(depreciation)
    }
}

// ============ SYD (Sum of Years Digits) ============

pub struct Syd;

static SYD_ARGS: [ArgMeta; 4] = [
    ArgMeta {
        name: "cost",
        typ: "Number",
        description: "Initial cost of asset",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "salvage",
        typ: "Number",
        description: "Salvage value at end of life",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "life",
        typ: "Number",
        description: "Useful life in periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "period",
        typ: "Number",
        description: "Period to calculate depreciation for",
        optional: false,
        default: None,
    },
];

static SYD_EXAMPLES: [&str; 1] = ["syd(100000, 10000, 10, 1) → 16363.64"];

static SYD_RELATED: [&str; 3] = ["sln", "ddb", "vdb"];

impl FunctionPlugin for Syd {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "syd",
            description: "Sum-of-years-digits depreciation",
            usage: "syd(cost, salvage, life, period)",
            args: &SYD_ARGS,
            returns: "Number",
            examples: &SYD_EXAMPLES,
            category: "finance/depreciation",
            source: None,
            related: &SYD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 4 {
            return Value::Error(FolioError::arg_count("syd", 4, args.len()));
        }

        let cost = match extract_number(&args[0], "syd", "cost") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let salvage = match extract_number(&args[1], "syd", "salvage") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let life = match extract_number(&args[2], "syd", "life") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let period = match extract_number(&args[3], "syd", "period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };

        if life <= 0 {
            return Value::Error(FolioError::domain_error(format!("syd: life must be positive, got {}", life)));
        }
        if period < 1 || period > life {
            return Value::Error(FolioError::domain_error(
                format!("syd: period must be between 1 and {}, got {}", life, period),
            ));
        }

        // Sum of years = n * (n + 1) / 2
        let sum_of_years = life * (life + 1) / 2;

        // Remaining years for this period
        let remaining = life - period + 1;

        // Depreciation = (cost - salvage) * remaining / sum_of_years
        let depreciable = cost.sub(&salvage);
        let fraction = Number::from_ratio(remaining, sum_of_years);
        let depreciation = depreciable.mul(&fraction);

        Value::Number(depreciation)
    }
}

// ============ VDB (Variable Declining Balance) ============

pub struct Vdb;

static VDB_ARGS: [ArgMeta; 7] = [
    ArgMeta {
        name: "cost",
        typ: "Number",
        description: "Initial cost of asset",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "salvage",
        typ: "Number",
        description: "Salvage value at end of life",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "life",
        typ: "Number",
        description: "Useful life in periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start_period",
        typ: "Number",
        description: "Starting period (0-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end_period",
        typ: "Number",
        description: "Ending period (exclusive)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "factor",
        typ: "Number",
        description: "Depreciation factor",
        optional: true,
        default: Some("2"),
    },
    ArgMeta {
        name: "no_switch",
        typ: "Bool",
        description: "If true, never switch to straight-line",
        optional: true,
        default: Some("false"),
    },
];

static VDB_EXAMPLES: [&str; 1] = ["vdb(100000, 10000, 10, 0, 2) → 36000"];

static VDB_RELATED: [&str; 2] = ["ddb", "sln"];

impl FunctionPlugin for Vdb {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "vdb",
            description: "Variable declining balance with optional switch to straight-line",
            usage: "vdb(cost, salvage, life, start_period, end_period, [factor], [no_switch])",
            args: &VDB_ARGS,
            returns: "Number",
            examples: &VDB_EXAMPLES,
            category: "finance/depreciation",
            source: None,
            related: &VDB_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 5 {
            return Value::Error(FolioError::arg_count("vdb", 5, args.len()));
        }

        let cost = match extract_number(&args[0], "vdb", "cost") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let salvage = match extract_number(&args[1], "vdb", "salvage") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let life = match extract_number(&args[2], "vdb", "life") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let start = match extract_number(&args[3], "vdb", "start_period") {
            Ok(n) => n.to_i64().unwrap_or(0),
            Err(e) => return Value::Error(e),
        };
        let end = match extract_number(&args[4], "vdb", "end_period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let factor = extract_optional_number(args, 5).unwrap_or_else(|| Number::from_i64(2));
        let no_switch = match args.get(6) {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };

        if life.is_zero() {
            return Value::Error(FolioError::domain_error("vdb: life must be positive"));
        }

        match calculate_vdb(&cost, &salvage, &life, start, end, &factor, no_switch) {
            Ok(d) => Value::Number(d),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_vdb(
    cost: &Number,
    salvage: &Number,
    life: &Number,
    start: i64,
    end: i64,
    factor: &Number,
    no_switch: bool,
) -> Result<Number, FolioError> {
    let life_i64 = life.to_i64().unwrap_or(1);
    let rate = factor.checked_div(life)?;

    let mut book_value = cost.clone();
    let mut total_depreciation = Number::from_i64(0);

    for period in 0..end {
        // Calculate DDB depreciation
        let ddb_dep = book_value.mul(&rate);

        // Calculate straight-line depreciation for remaining periods
        let remaining_periods = life_i64 - period;
        let sln_dep = if remaining_periods > 0 {
            book_value.sub(salvage).checked_div(&Number::from_i64(remaining_periods))?
        } else {
            Number::from_i64(0)
        };

        // Use the larger unless no_switch is true
        let depreciation = if no_switch {
            ddb_dep
        } else if &sln_dep > &ddb_dep {
            sln_dep
        } else {
            ddb_dep
        };

        // Don't depreciate below salvage
        let max_dep = book_value.sub(salvage);
        let actual_dep = if &depreciation > &max_dep && !max_dep.is_negative() {
            max_dep
        } else if max_dep.is_negative() {
            Number::from_i64(0)
        } else {
            depreciation
        };

        if period >= start {
            total_depreciation = total_depreciation.add(&actual_dep);
        }

        book_value = book_value.sub(&actual_dep);
    }

    Ok(total_depreciation)
}

// ============ Depreciation Schedule ============

pub struct DepreciationSchedule;

static DEP_SCHED_ARGS: [ArgMeta; 4] = [
    ArgMeta {
        name: "cost",
        typ: "Number",
        description: "Initial cost of asset",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "salvage",
        typ: "Number",
        description: "Salvage value at end of life",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "life",
        typ: "Number",
        description: "Useful life in periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "method",
        typ: "Text",
        description: "Method: 'sln', 'ddb', 'syd', 'ddb150'",
        optional: false,
        default: None,
    },
];

static DEP_SCHED_EXAMPLES: [&str; 1] = ["depreciation_schedule(100000, 10000, 10, \"ddb\")"];

static DEP_SCHED_RELATED: [&str; 4] = ["sln", "ddb", "syd", "vdb"];

impl FunctionPlugin for DepreciationSchedule {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "depreciation_schedule",
            description: "Full depreciation schedule",
            usage: "depreciation_schedule(cost, salvage, life, method)",
            args: &DEP_SCHED_ARGS,
            returns: "Object",
            examples: &DEP_SCHED_EXAMPLES,
            category: "finance/depreciation",
            source: None,
            related: &DEP_SCHED_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 4 {
            return Value::Error(FolioError::arg_count("depreciation_schedule", 4, args.len()));
        }

        let cost = match extract_number(&args[0], "depreciation_schedule", "cost") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let salvage = match extract_number(&args[1], "depreciation_schedule", "salvage") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let life = match extract_number(&args[2], "depreciation_schedule", "life") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let method = match &args[3] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "depreciation_schedule",
                "method",
                "Text",
                other.type_name(),
            )),
        };

        if life <= 0 {
            return Value::Error(FolioError::domain_error(
                format!("depreciation_schedule: life must be positive, got {}", life),
            ));
        }

        let mut schedule = Vec::new();
        let mut book_value = cost.clone();
        let mut total_depreciation = Number::from_i64(0);

        let life_num = Number::from_i64(life);
        let factor_150 = Number::from_str("1.5").unwrap();
        let factor_200 = Number::from_i64(2);

        for period in 1..=life {
            let depreciation = match method.as_str() {
                "sln" => cost.sub(&salvage).checked_div(&life_num).unwrap_or_else(|_| Number::from_i64(0)),
                "ddb" => calculate_ddb(&cost, &salvage, &life_num, period, &factor_200).unwrap_or_else(|_| Number::from_i64(0)),
                "ddb150" => calculate_ddb(&cost, &salvage, &life_num, period, &factor_150).unwrap_or_else(|_| Number::from_i64(0)),
                "syd" => {
                    let sum_of_years = life * (life + 1) / 2;
                    let remaining = life - period + 1;
                    cost.sub(&salvage).mul(&Number::from_ratio(remaining, sum_of_years))
                }
                _ => return Value::Error(FolioError::domain_error(
                    format!("depreciation_schedule: Unknown method '{}'. Use 'sln', 'ddb', 'syd', or 'ddb150'", method),
                )),
            };

            // Don't go below salvage
            let max_dep = book_value.sub(&salvage);
            let actual_dep = if &depreciation > &max_dep && !max_dep.is_negative() {
                max_dep
            } else if max_dep.is_negative() {
                Number::from_i64(0)
            } else {
                depreciation
            };

            book_value = book_value.sub(&actual_dep);
            total_depreciation = total_depreciation.add(&actual_dep);

            let mut row = HashMap::new();
            row.insert("period".to_string(), Value::Number(Number::from_i64(period)));
            row.insert("depreciation".to_string(), Value::Number(actual_dep));
            row.insert("book_value".to_string(), Value::Number(book_value.clone()));
            schedule.push(Value::Object(row));
        }

        let mut result = HashMap::new();
        result.insert("method".to_string(), Value::Text(method));
        result.insert("cost".to_string(), Value::Number(cost));
        result.insert("salvage".to_string(), Value::Number(salvage));
        result.insert("life".to_string(), Value::Number(Number::from_i64(life)));
        result.insert("schedule".to_string(), Value::List(schedule));
        result.insert("total_depreciation".to_string(), Value::Number(total_depreciation));

        Value::Object(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_sln() {
        let f = Sln;
        let args = vec![
            Value::Number(Number::from_i64(100000)),
            Value::Number(Number::from_i64(10000)),
            Value::Number(Number::from_i64(10)),
        ];
        let result = f.call(&args, &eval_ctx());
        let dep = result.as_number().unwrap();
        assert_eq!(dep.to_i64(), Some(9000));
    }

    #[test]
    fn test_ddb_year1() {
        let f = Ddb;
        let args = vec![
            Value::Number(Number::from_i64(100000)),
            Value::Number(Number::from_i64(10000)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(1)),
        ];
        let result = f.call(&args, &eval_ctx());
        let dep = result.as_number().unwrap();
        assert_eq!(dep.to_i64(), Some(20000));
    }

    #[test]
    fn test_syd_year1() {
        let f = Syd;
        let args = vec![
            Value::Number(Number::from_i64(100000)),
            Value::Number(Number::from_i64(10000)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(1)),
        ];
        let result = f.call(&args, &eval_ctx());
        let dep = result.as_number().unwrap();
        // (100000 - 10000) * 10 / 55 = 16363.64
        let expected = Number::from_str("16363.63").unwrap();
        let diff = dep.sub(&expected).abs();
        assert!(diff < Number::from_str("1").unwrap());
    }

    #[test]
    fn test_vdb() {
        let f = Vdb;
        let args = vec![
            Value::Number(Number::from_i64(100000)),
            Value::Number(Number::from_i64(10000)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(2)),
        ];
        let result = f.call(&args, &eval_ctx());
        let dep = result.as_number().unwrap();
        // Year 1: 20000, Year 2: 16000, Total: 36000
        assert_eq!(dep.to_i64(), Some(36000));
    }
}
