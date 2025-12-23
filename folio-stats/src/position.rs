//! Position functions: min, max, percentile, quantile, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, mean, variance_impl, sorted, percentile_impl, ranks};

// ============ Min ============

pub struct Min;

static MIN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static MIN_EXAMPLES: [&str; 1] = ["min(3, 1, 4, 1, 5) → 1"];

static MIN_RELATED: [&str; 2] = ["max", "range"];

impl FunctionPlugin for Min {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "min",
            description: "Minimum value",
            usage: "min(values)",
            args: &MIN_ARGS,
            returns: "Number",
            examples: &MIN_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &MIN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "min") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        Value::Number(sorted_nums[0].clone())
    }
}

// ============ Max ============

pub struct Max;

static MAX_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static MAX_EXAMPLES: [&str; 1] = ["max(3, 1, 4, 1, 5) → 5"];

static MAX_RELATED: [&str; 2] = ["min", "range"];

impl FunctionPlugin for Max {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "max",
            description: "Maximum value",
            usage: "max(values)",
            args: &MAX_ARGS,
            returns: "Number",
            examples: &MAX_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &MAX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "max") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        Value::Number(sorted_nums[sorted_nums.len() - 1].clone())
    }
}

// ============ Percentile ============

pub struct Percentile;

static PERCENTILE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Numbers",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Percentile (0-100)",
        optional: false,
        default: None,
    },
];

static PERCENTILE_EXAMPLES: [&str; 1] = ["percentile([1,2,3,4,5], 50) → 3"];

static PERCENTILE_RELATED: [&str; 3] = ["quantile", "median", "q1"];

impl FunctionPlugin for Percentile {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "percentile",
            description: "p-th percentile (0-100)",
            usage: "percentile(values, p)",
            args: &PERCENTILE_ARGS,
            returns: "Number",
            examples: &PERCENTILE_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &PERCENTILE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("percentile", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let p = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("percentile", "p", "Number", other.type_name())),
        };

        match percentile_impl(&numbers, p) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Quantile ============

pub struct Quantile;

static QUANTILE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Numbers",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "q",
        typ: "Number",
        description: "Quantile (0-1)",
        optional: false,
        default: None,
    },
];

static QUANTILE_EXAMPLES: [&str; 1] = ["quantile([1,2,3,4,5], 0.5) → 3"];

static QUANTILE_RELATED: [&str; 2] = ["percentile", "median"];

impl FunctionPlugin for Quantile {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "quantile",
            description: "q-th quantile (0-1)",
            usage: "quantile(values, q)",
            args: &QUANTILE_ARGS,
            returns: "Number",
            examples: &QUANTILE_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &QUANTILE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("quantile", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let q = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("quantile", "q", "Number", other.type_name())),
        };

        // Validate q is in [0, 1]
        let q_f64 = q.to_f64().unwrap_or(0.0);
        if q_f64 < 0.0 || q_f64 > 1.0 {
            return Value::Error(FolioError::domain_error(
                "quantile() requires 0 <= q <= 1",
            ));
        }

        // Convert quantile to percentile
        let hundred = Number::from_i64(100);
        let p = q.mul(&hundred);

        match percentile_impl(&numbers, &p) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Q1 ============

pub struct Q1;

static Q1_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static Q1_EXAMPLES: [&str; 1] = ["q1([1,2,3,4,5,6,7,8]) → 2.5"];

static Q1_RELATED: [&str; 3] = ["q3", "median", "iqr"];

impl FunctionPlugin for Q1 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "q1",
            description: "First quartile (25th percentile)",
            usage: "q1(values)",
            args: &Q1_ARGS,
            returns: "Number",
            examples: &Q1_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &Q1_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let p25 = Number::from_i64(25);
        match percentile_impl(&numbers, &p25) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Q3 ============

pub struct Q3;

static Q3_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static Q3_EXAMPLES: [&str; 1] = ["q3([1,2,3,4,5,6,7,8]) → 6.5"];

static Q3_RELATED: [&str; 3] = ["q1", "median", "iqr"];

impl FunctionPlugin for Q3 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "q3",
            description: "Third quartile (75th percentile)",
            usage: "q3(values)",
            args: &Q3_ARGS,
            returns: "Number",
            examples: &Q3_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &Q3_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let p75 = Number::from_i64(75);
        match percentile_impl(&numbers, &p75) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Rank ============

pub struct Rank;

static RANK_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to find rank of",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Reference list",
        optional: false,
        default: None,
    },
];

static RANK_EXAMPLES: [&str; 1] = ["rank(3, [1,2,3,4,5]) → 3"];

static RANK_RELATED: [&str; 2] = ["percentile", "zscore"];

impl FunctionPlugin for Rank {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "rank",
            description: "Position in sorted list (1-indexed)",
            usage: "rank(value, list)",
            args: &RANK_ARGS,
            returns: "Number",
            examples: &RANK_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &RANK_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("rank", 2, args.len()));
        }

        let value = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("rank", "value", "Number", other.type_name())),
        };

        let numbers = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "rank") {
            return Value::Error(e);
        }

        // Add the value to the list and compute ranks
        let mut all_values = numbers.clone();
        all_values.push(value.clone());
        let all_ranks = ranks(&all_values);

        // Return the rank of the last element (our value)
        Value::Number(all_ranks[all_values.len() - 1].clone())
    }
}

// ============ Zscore ============

pub struct Zscore;

static ZSCORE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to standardize",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Reference list",
        optional: false,
        default: None,
    },
];

static ZSCORE_EXAMPLES: [&str; 1] = ["zscore(5, [2,4,4,4,5,5,7,9]) → 0.467..."];

static ZSCORE_RELATED: [&str; 2] = ["normalize", "stddev"];

impl FunctionPlugin for Zscore {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "zscore",
            description: "Standard score: (x - mean)/stddev",
            usage: "zscore(value, list)",
            args: &ZSCORE_ARGS,
            returns: "Number",
            examples: &ZSCORE_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &ZSCORE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("zscore", 2, args.len()));
        }

        let value = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("zscore", "value", "Number", other.type_name())),
        };

        let numbers = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if numbers.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "zscore() requires at least 2 values in list",
            ));
        }

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd.is_zero() {
            return Value::Error(FolioError::domain_error(
                "zscore() undefined when stddev is zero",
            ));
        }

        let deviation = value.sub(&m);
        match deviation.checked_div(&sd) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_min() {
        let min = Min;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(4)),
        ])];
        let result = min.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }

    #[test]
    fn test_max() {
        let max = Max;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(4)),
        ])];
        let result = max.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_percentile() {
        let percentile = Percentile;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(5)),
            ]),
            Value::Number(Number::from_i64(50)),
        ];
        let result = percentile.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }
}
