//! Transform functions: normalize, standardize, cumsum, differences, lag, moving_avg, ewma

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, require_min_count, mean, variance_impl, sorted};

// ============ Normalize (Z-scores) ============

pub struct Normalize;

static NORMALIZE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to normalize",
    optional: false,
    default: None,
}];

static NORMALIZE_EXAMPLES: [&str; 1] = ["normalize([1,2,3,4,5]) → z-scores"];

static NORMALIZE_RELATED: [&str; 2] = ["standardize", "zscore"];

impl FunctionPlugin for Normalize {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "normalize",
            description: "Z-scores for all values: (x - mean)/stddev",
            usage: "normalize(list)",
            args: &NORMALIZE_ARGS,
            returns: "List<Number>",
            examples: &NORMALIZE_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &NORMALIZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "normalize") {
            return Value::Error(e);
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
                "normalize() requires non-zero standard deviation",
            ));
        }

        let normalized: Vec<Value> = numbers
            .iter()
            .map(|x| {
                let z = x.sub(&m).checked_div(&sd).unwrap_or(Number::from_i64(0));
                Value::Number(z)
            })
            .collect();

        Value::List(normalized)
    }
}

// ============ Standardize (0-1 range) ============

pub struct Standardize;

static STANDARDIZE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to standardize",
    optional: false,
    default: None,
}];

static STANDARDIZE_EXAMPLES: [&str; 1] = ["standardize([1,2,3,4,5]) → [0, 0.25, 0.5, 0.75, 1]"];

static STANDARDIZE_RELATED: [&str; 2] = ["normalize", "range"];

impl FunctionPlugin for Standardize {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "standardize",
            description: "Scale to [0,1] range: (x - min)/(max - min)",
            usage: "standardize(list)",
            args: &STANDARDIZE_ARGS,
            returns: "List<Number>",
            examples: &STANDARDIZE_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &STANDARDIZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "standardize") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        let min = sorted_nums[0].clone();
        let max = sorted_nums[sorted_nums.len() - 1].clone();
        let range = max.sub(&min);

        if range.is_zero() {
            // All values are equal
            return Value::List(numbers.iter().map(|_| Value::Number(Number::from_i64(0))).collect());
        }

        let standardized: Vec<Value> = numbers
            .iter()
            .map(|x| {
                let scaled = x.sub(&min).checked_div(&range).unwrap_or(Number::from_i64(0));
                Value::Number(scaled)
            })
            .collect();

        Value::List(standardized)
    }
}

// ============ Cumsum ============

pub struct Cumsum;

static CUMSUM_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to sum cumulatively",
    optional: false,
    default: None,
}];

static CUMSUM_EXAMPLES: [&str; 1] = ["cumsum([1,2,3,4,5]) → [1,3,6,10,15]"];

static CUMSUM_RELATED: [&str; 2] = ["sum", "differences"];

impl FunctionPlugin for Cumsum {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cumsum",
            description: "Cumulative sum",
            usage: "cumsum(list)",
            args: &CUMSUM_ARGS,
            returns: "List<Number>",
            examples: &CUMSUM_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &CUMSUM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let mut sum = Number::from_i64(0);
        let cumulative: Vec<Value> = numbers
            .iter()
            .map(|x| {
                sum = sum.add(x);
                Value::Number(sum.clone())
            })
            .collect();

        Value::List(cumulative)
    }
}

// ============ Differences ============

pub struct Differences;

static DIFFERENCES_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to difference",
    optional: false,
    default: None,
}];

static DIFFERENCES_EXAMPLES: [&str; 1] = ["differences([1,3,6,10,15]) → [2,3,4,5]"];

static DIFFERENCES_RELATED: [&str; 2] = ["cumsum", "lag"];

impl FunctionPlugin for Differences {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "differences",
            description: "First differences: x[i] - x[i-1]",
            usage: "differences(list)",
            args: &DIFFERENCES_ARGS,
            returns: "List<Number>",
            examples: &DIFFERENCES_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &DIFFERENCES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if numbers.len() < 2 {
            return Value::List(vec![]);
        }

        let diffs: Vec<Value> = numbers
            .windows(2)
            .map(|w| Value::Number(w[1].sub(&w[0])))
            .collect();

        Value::List(diffs)
    }
}

// ============ Lag ============

pub struct Lag;

static LAG_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values to lag",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of periods to lag (default 1)",
        optional: true,
        default: Some("1"),
    },
];

static LAG_EXAMPLES: [&str; 1] = ["lag([1,2,3,4,5], 2) → [null,null,1,2,3]"];

static LAG_RELATED: [&str; 2] = ["differences", "moving_avg"];

impl FunctionPlugin for Lag {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lag",
            description: "Shift by n periods (earlier values become null)",
            usage: "lag(list, n?)",
            args: &LAG_ARGS,
            returns: "List<Number|Null>",
            examples: &LAG_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &LAG_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("lag", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = if args.len() > 1 {
            match &args[1] {
                Value::Number(num) => num.to_i64().unwrap_or(1) as usize,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("lag", "n", "Number", other.type_name())),
            }
        } else {
            1
        };

        let mut lagged: Vec<Value> = Vec::with_capacity(numbers.len());

        for i in 0..numbers.len() {
            if i < n {
                lagged.push(Value::Null);
            } else {
                lagged.push(Value::Number(numbers[i - n].clone()));
            }
        }

        Value::List(lagged)
    }
}

// ============ Moving Average ============

pub struct MovingAvg;

static MOVING_AVG_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "window",
        typ: "Number",
        description: "Window size",
        optional: false,
        default: None,
    },
];

static MOVING_AVG_EXAMPLES: [&str; 1] = ["moving_avg([1,2,3,4,5], 3) → [null,null,2,3,4]"];

static MOVING_AVG_RELATED: [&str; 2] = ["ewma", "mean"];

impl FunctionPlugin for MovingAvg {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "moving_avg",
            description: "Simple moving average",
            usage: "moving_avg(list, window)",
            args: &MOVING_AVG_ARGS,
            returns: "List<Number|Null>",
            examples: &MOVING_AVG_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &MOVING_AVG_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("moving_avg", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let window = match &args[1] {
            Value::Number(n) => n.to_i64().unwrap_or(1) as usize,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("moving_avg", "window", "Number", other.type_name())),
        };

        if window == 0 {
            return Value::Error(FolioError::domain_error(
                "moving_avg() requires window > 0",
            ));
        }

        let mut result: Vec<Value> = Vec::with_capacity(numbers.len());
        let window_size = Number::from_i64(window as i64);

        for i in 0..numbers.len() {
            if i + 1 < window {
                result.push(Value::Null);
            } else {
                let start = i + 1 - window;
                let sum: Number = numbers[start..=i]
                    .iter()
                    .fold(Number::from_i64(0), |acc, x| acc.add(x));
                let avg = sum.checked_div(&window_size).unwrap_or(Number::from_i64(0));
                result.push(Value::Number(avg));
            }
        }

        Value::List(result)
    }
}

// ============ Exponentially Weighted Moving Average ============

pub struct Ewma;

static EWMA_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "alpha",
        typ: "Number",
        description: "Smoothing factor (0 < α ≤ 1)",
        optional: false,
        default: None,
    },
];

static EWMA_EXAMPLES: [&str; 1] = ["ewma([1,2,3,4,5], 0.3)"];

static EWMA_RELATED: [&str; 2] = ["moving_avg", "mean"];

impl FunctionPlugin for Ewma {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ewma",
            description: "Exponentially weighted moving average",
            usage: "ewma(list, alpha)",
            args: &EWMA_ARGS,
            returns: "List<Number>",
            examples: &EWMA_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &EWMA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("ewma", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let alpha = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("ewma", "alpha", "Number", other.type_name())),
        };

        let alpha_f64 = alpha.to_f64().unwrap_or(0.0);
        if alpha_f64 <= 0.0 || alpha_f64 > 1.0 {
            return Value::Error(FolioError::domain_error(
                "ewma() requires 0 < alpha ≤ 1",
            ));
        }

        if numbers.is_empty() {
            return Value::List(vec![]);
        }

        let one_minus_alpha = Number::from_i64(1).sub(alpha);
        let mut ewma = numbers[0].clone();
        let mut result: Vec<Value> = vec![Value::Number(ewma.clone())];

        for x in numbers.iter().skip(1) {
            // EWMA = α * x + (1-α) * EWMA_prev
            ewma = alpha.mul(x).add(&one_minus_alpha.mul(&ewma));
            result.push(Value::Number(ewma.clone()));
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
    fn test_cumsum() {
        let cumsum = Cumsum;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = cumsum.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(15));
    }

    #[test]
    fn test_differences() {
        let differences = Differences;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(6)),
            Value::Number(Number::from_i64(10)),
        ])];
        let result = differences.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_standardize() {
        let standardize = Standardize;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(50)),
            Value::Number(Number::from_i64(100)),
        ])];
        let result = standardize.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(0));
        // list[1] should be 0.5
        // list[2] should be 1
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(1));
    }
}
