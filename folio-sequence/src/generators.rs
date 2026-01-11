//! Basic sequence generators
//!
//! range, linspace, arithmetic, geometric, harmonic, repeat, cycle

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_optional_number, extract_list, require_count};

// ============ Range ============

pub struct Range;

static RANGE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Starting value (inclusive)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "Ending value (inclusive)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "step",
        typ: "Number",
        description: "Step increment",
        optional: true,
        default: Some("1"),
    },
];

static RANGE_EXAMPLES: [&str; 3] = [
    "range(1, 5) → [1, 2, 3, 4, 5]",
    "range(0, 10, 2) → [0, 2, 4, 6, 8, 10]",
    "range(10, 1, -1) → [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]",
];

static RANGE_RELATED: [&str; 2] = ["linspace", "arithmetic"];

impl FunctionPlugin for Range {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "range",
            description: "Generate integer sequence from start to end (inclusive)",
            usage: "range(start, end, [step])",
            args: &RANGE_ARGS,
            returns: "List<Number>",
            examples: &RANGE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &RANGE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("range", 2, args.len()));
        }

        let start = match extract_number(&args[0], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[1], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let step = match extract_optional_number(args, 2) {
            Ok(Some(n)) => n,
            Ok(None) => {
                // Default step: 1 if start <= end, -1 if start > end
                if start.sub(&end).is_negative() || start.sub(&end).is_zero() {
                    Number::from_i64(1)
                } else {
                    Number::from_i64(-1)
                }
            }
            Err(e) => return Value::Error(e),
        };

        if step.is_zero() {
            return Value::Error(FolioError::domain_error("range() step cannot be zero"));
        }

        let mut result = Vec::new();
        let mut current = start.clone();
        let zero = Number::from_i64(0);
        let max_elements = 100000;

        if step.sub(&zero).is_negative() {
            // Descending
            while current.sub(&end).is_negative() == false && !current.sub(&end).is_zero() || current == end {
                if result.len() >= max_elements {
                    return Value::Error(FolioError::domain_error(
                        format!("range() would generate more than {} elements", max_elements)
                    ));
                }
                result.push(Value::Number(current.clone()));
                current = current.add(&step);
                if current.sub(&end).is_negative() {
                    break;
                }
            }
        } else {
            // Ascending
            while current.sub(&end).is_negative() || current == end {
                if result.len() >= max_elements {
                    return Value::Error(FolioError::domain_error(
                        format!("range() would generate more than {} elements", max_elements)
                    ));
                }
                result.push(Value::Number(current.clone()));
                current = current.add(&step);
            }
        }

        Value::List(result)
    }
}

// ============ Linspace ============

pub struct Linspace;

static LINSPACE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Starting value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "Ending value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of points (including endpoints)",
        optional: false,
        default: None,
    },
];

static LINSPACE_EXAMPLES: [&str; 1] = [
    "linspace(0, 1, 5) → [0, 0.25, 0.5, 0.75, 1]",
];

static LINSPACE_RELATED: [&str; 2] = ["range", "logspace"];

impl FunctionPlugin for Linspace {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "linspace",
            description: "Generate linearly spaced values (includes endpoints)",
            usage: "linspace(start, end, count)",
            args: &LINSPACE_ARGS,
            returns: "List<Number>",
            examples: &LINSPACE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &LINSPACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("linspace", 3, args.len()));
        }

        let start = match extract_number(&args[0], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[1], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "linspace", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        if count == 1 {
            return Value::List(vec![Value::Number(start)]);
        }

        let mut result = Vec::with_capacity(count);
        let range = end.sub(&start);
        let divisor = Number::from_i64((count - 1) as i64);

        for i in 0..count {
            let t = Number::from_i64(i as i64);
            let fraction = match t.checked_div(&divisor) {
                Ok(f) => f,
                Err(e) => return Value::Error(e.into()),
            };
            let value = start.add(&range.mul(&fraction));
            result.push(Value::Number(value));
        }

        Value::List(result)
    }
}

// ============ Logspace ============

pub struct Logspace;

static LOGSPACE_ARGS: [ArgMeta; 4] = [
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Starting value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "Ending value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of points",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "base",
        typ: "Number",
        description: "Logarithm base",
        optional: true,
        default: Some("10"),
    },
];

static LOGSPACE_EXAMPLES: [&str; 2] = [
    "logspace(1, 1000, 4) → [1, 10, 100, 1000]",
    "logspace(1, 8, 4, 2) → [1, 2, 4, 8]",
];

static LOGSPACE_RELATED: [&str; 2] = ["linspace", "geometric"];

impl FunctionPlugin for Logspace {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "logspace",
            description: "Generate logarithmically spaced values",
            usage: "logspace(start, end, count, [base])",
            args: &LOGSPACE_ARGS,
            returns: "List<Number>",
            examples: &LOGSPACE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &LOGSPACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 || args.len() > 4 {
            return Value::Error(FolioError::arg_count("logspace", 3, args.len()));
        }

        let start = match extract_number(&args[0], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[1], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "logspace", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let base = match extract_optional_number(args, 3) {
            Ok(Some(n)) => n,
            Ok(None) => Number::from_i64(10),
            Err(e) => return Value::Error(e),
        };

        // Validate start and end are positive
        if start.is_zero() || start.is_negative() || end.is_zero() || end.is_negative() {
            return Value::Error(FolioError::domain_error(
                "logspace() requires positive start and end values"
            ));
        }

        if count == 1 {
            return Value::List(vec![Value::Number(start)]);
        }

        let mut result = Vec::with_capacity(count);

        // Calculate log(start) and log(end) in the given base
        // log_b(x) = ln(x) / ln(b)
        let ln_base = match base.ln(ctx.precision) {
            Ok(ln) => ln,
            Err(e) => return Value::Error(e.into()),
        };

        let log_start = match start.ln(ctx.precision) {
            Ok(ln) => match ln.checked_div(&ln_base) {
                Ok(l) => l,
                Err(e) => return Value::Error(e.into()),
            },
            Err(e) => return Value::Error(e.into()),
        };

        let log_end = match end.ln(ctx.precision) {
            Ok(ln) => match ln.checked_div(&ln_base) {
                Ok(l) => l,
                Err(e) => return Value::Error(e.into()),
            },
            Err(e) => return Value::Error(e.into()),
        };

        let log_range = log_end.sub(&log_start);
        let divisor = Number::from_i64((count - 1) as i64);

        for i in 0..count {
            let t = Number::from_i64(i as i64);
            let fraction = match t.checked_div(&divisor) {
                Ok(f) => f,
                Err(e) => return Value::Error(e.into()),
            };
            let log_val = log_start.add(&log_range.mul(&fraction));
            // value = base^log_val
            let value = base.pow_real(&log_val, ctx.precision);
            result.push(Value::Number(value));
        }

        Value::List(result)
    }
}

// ============ Arithmetic ============

pub struct Arithmetic;

static ARITHMETIC_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "first",
        typ: "Number",
        description: "First term",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "diff",
        typ: "Number",
        description: "Common difference",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of terms",
        optional: false,
        default: None,
    },
];

static ARITHMETIC_EXAMPLES: [&str; 1] = [
    "arithmetic(5, 3, 6) → [5, 8, 11, 14, 17, 20]",
];

static ARITHMETIC_RELATED: [&str; 2] = ["geometric", "range"];

impl FunctionPlugin for Arithmetic {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "arithmetic",
            description: "Generate arithmetic sequence: a_n = first + (n-1) × diff",
            usage: "arithmetic(first, diff, count)",
            args: &ARITHMETIC_ARGS,
            returns: "List<Number>",
            examples: &ARITHMETIC_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &ARITHMETIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("arithmetic", 3, args.len()));
        }

        let first = match extract_number(&args[0], "first") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let diff = match extract_number(&args[1], "diff") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "arithmetic", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut current = first;

        for _ in 0..count {
            result.push(Value::Number(current.clone()));
            current = current.add(&diff);
        }

        Value::List(result)
    }
}

// ============ Geometric ============

pub struct Geometric;

static GEOMETRIC_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "first",
        typ: "Number",
        description: "First term",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "ratio",
        typ: "Number",
        description: "Common ratio",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of terms",
        optional: false,
        default: None,
    },
];

static GEOMETRIC_EXAMPLES: [&str; 1] = [
    "geometric(2, 3, 5) → [2, 6, 18, 54, 162]",
];

static GEOMETRIC_RELATED: [&str; 2] = ["arithmetic", "powers"];

impl FunctionPlugin for Geometric {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "geometric",
            description: "Generate geometric sequence: a_n = first × ratio^(n-1)",
            usage: "geometric(first, ratio, count)",
            args: &GEOMETRIC_ARGS,
            returns: "List<Number>",
            examples: &GEOMETRIC_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &GEOMETRIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("geometric", 3, args.len()));
        }

        let first = match extract_number(&args[0], "first") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let ratio = match extract_number(&args[1], "ratio") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "geometric", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut current = first;

        for _ in 0..count {
            result.push(Value::Number(current.clone()));
            current = current.mul(&ratio);
        }

        Value::List(result)
    }
}

// ============ Harmonic ============

pub struct Harmonic;

static HARMONIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of terms",
    optional: false,
    default: None,
}];

static HARMONIC_EXAMPLES: [&str; 1] = [
    "harmonic(5) → [1, 0.5, 0.333..., 0.25, 0.2]",
];

static HARMONIC_RELATED: [&str; 1] = ["arithmetic"];

impl FunctionPlugin for Harmonic {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "harmonic",
            description: "Generate harmonic sequence: 1, 1/2, 1/3, 1/4, ...",
            usage: "harmonic(count)",
            args: &HARMONIC_ARGS,
            returns: "List<Number>",
            examples: &HARMONIC_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &HARMONIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("harmonic", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "harmonic", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let one = Number::from_i64(1);
        let mut result = Vec::with_capacity(count);

        for i in 1..=count {
            let denom = Number::from_i64(i as i64);
            match one.checked_div(&denom) {
                Ok(val) => result.push(Value::Number(val)),
                Err(e) => return Value::Error(e.into()),
            }
        }

        Value::List(result)
    }
}

// ============ RepeatSeq ============

pub struct RepeatSeq;

static REPEAT_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to repeat",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of repetitions",
        optional: false,
        default: None,
    },
];

static REPEAT_SEQ_EXAMPLES: [&str; 1] = [
    "repeat_seq(7, 5) → [7, 7, 7, 7, 7]",
];

static REPEAT_SEQ_RELATED: [&str; 1] = ["cycle"];

impl FunctionPlugin for RepeatSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "repeat_seq",
            description: "Repeat a single value",
            usage: "repeat_seq(value, count)",
            args: &REPEAT_SEQ_ARGS,
            returns: "List<Number>",
            examples: &REPEAT_SEQ_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &REPEAT_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("repeat_seq", 2, args.len()));
        }

        let value = match extract_number(&args[0], "value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "repeat_seq", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let result: Vec<Value> = (0..count).map(|_| Value::Number(value.clone())).collect();
        Value::List(result)
    }
}

// ============ Cycle ============

pub struct Cycle;

static CYCLE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "List to cycle through",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Total number of elements to generate",
        optional: false,
        default: None,
    },
];

static CYCLE_EXAMPLES: [&str; 1] = [
    "cycle([1, 2, 3], 8) → [1, 2, 3, 1, 2, 3, 1, 2]",
];

static CYCLE_RELATED: [&str; 1] = ["repeat_seq"];

impl FunctionPlugin for Cycle {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cycle",
            description: "Cycle through a list to generate count elements",
            usage: "cycle(list, count)",
            args: &CYCLE_ARGS,
            returns: "List<Number>",
            examples: &CYCLE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &CYCLE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("cycle", 2, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.is_empty() {
            return Value::Error(FolioError::domain_error("cycle() requires non-empty list"));
        }

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "cycle", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            result.push(Value::Number(list[i % list.len()].clone()));
        }

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_range_basic() {
        let range = Range;
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(5)),
        ];
        let result = range.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(5));
    }

    #[test]
    fn test_range_with_step() {
        let range = Range;
        let args = vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(2)),
        ];
        let result = range.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 6); // 0, 2, 4, 6, 8, 10
    }

    #[test]
    fn test_arithmetic() {
        let arith = Arithmetic;
        let args = vec![
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ];
        let result = arith.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(8));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(11));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(14));
    }

    #[test]
    fn test_geometric() {
        let geom = Geometric;
        let args = vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ];
        let result = geom.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(6));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(18));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(54));
    }

    #[test]
    fn test_harmonic() {
        let harm = Harmonic;
        let args = vec![Value::Number(Number::from_i64(3))];
        let result = harm.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        // 1/2 and 1/3 won't be exact integers
    }

    #[test]
    fn test_cycle() {
        let cycle = Cycle;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::Number(Number::from_i64(7)),
        ];
        let result = cycle.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 7);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[6].as_number().unwrap().to_i64(), Some(1));
    }
}
