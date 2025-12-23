//! Dispersion functions: variance, stddev, range, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, require_min_count, mean, variance_impl, sorted, percentile_impl};

// ============ Variance (Sample) ============

pub struct Variance;

static VARIANCE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Sample data",
    optional: false,
    default: None,
}];

static VARIANCE_EXAMPLES: [&str; 2] = [
    "variance(2, 4, 4, 4, 5, 5, 7, 9) → 4.571...",
    "variance(Data.Value) → sample variance of column",
];

static VARIANCE_RELATED: [&str; 3] = ["variance_p", "stddev", "cv"];

impl FunctionPlugin for Variance {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "variance",
            description: "Sample variance (divides by n-1)",
            usage: "variance(values)",
            args: &VARIANCE_ARGS,
            returns: "Number",
            examples: &VARIANCE_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &VARIANCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, true) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Variance (Population) ============

pub struct VarianceP;

static VARIANCE_P_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Population data",
    optional: false,
    default: None,
}];

static VARIANCE_P_EXAMPLES: [&str; 1] = ["variance_p(2, 4, 4, 4, 5, 5, 7, 9) → 4"];

static VARIANCE_P_RELATED: [&str; 3] = ["variance", "stddev_p", "cv"];

impl FunctionPlugin for VarianceP {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "variance_p",
            description: "Population variance (divides by n)",
            usage: "variance_p(values)",
            args: &VARIANCE_P_ARGS,
            returns: "Number",
            examples: &VARIANCE_P_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &VARIANCE_P_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, false) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Stddev (Sample) ============

pub struct Stddev;

static STDDEV_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Sample data",
    optional: false,
    default: None,
}];

static STDDEV_EXAMPLES: [&str; 1] = ["stddev(2, 4, 4, 4, 5, 5, 7, 9) → 2.138..."];

static STDDEV_RELATED: [&str; 3] = ["stddev_p", "variance", "se"];

impl FunctionPlugin for Stddev {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "stddev",
            description: "Sample standard deviation: √variance",
            usage: "stddev(values)",
            args: &STDDEV_ARGS,
            returns: "Number",
            examples: &STDDEV_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &STDDEV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, true) {
            Ok(var) => match var.sqrt(ctx.precision) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            },
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Stddev (Population) ============

pub struct StddevP;

static STDDEV_P_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Population data",
    optional: false,
    default: None,
}];

static STDDEV_P_EXAMPLES: [&str; 1] = ["stddev_p(2, 4, 4, 4, 5, 5, 7, 9) → 2"];

static STDDEV_P_RELATED: [&str; 3] = ["stddev", "variance_p", "se"];

impl FunctionPlugin for StddevP {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "stddev_p",
            description: "Population standard deviation",
            usage: "stddev_p(values)",
            args: &STDDEV_P_ARGS,
            returns: "Number",
            examples: &STDDEV_P_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &STDDEV_P_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, false) {
            Ok(var) => match var.sqrt(ctx.precision) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            },
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Range ============

pub struct Range;

static RANGE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static RANGE_EXAMPLES: [&str; 1] = ["range(1, 5, 10) → 9"];

static RANGE_RELATED: [&str; 2] = ["min", "max"];

impl FunctionPlugin for Range {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "range",
            description: "max - min",
            usage: "range(values)",
            args: &RANGE_ARGS,
            returns: "Number",
            examples: &RANGE_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &RANGE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "range") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        let min = &sorted_nums[0];
        let max = &sorted_nums[sorted_nums.len() - 1];

        Value::Number(max.sub(min))
    }
}

// ============ IQR ============

pub struct Iqr;

static IQR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static IQR_EXAMPLES: [&str; 1] = ["iqr(1, 2, 3, 4, 5, 6, 7, 8) → 4"];

static IQR_RELATED: [&str; 3] = ["q1", "q3", "percentile"];

impl FunctionPlugin for Iqr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "iqr",
            description: "Interquartile range: Q3 - Q1",
            usage: "iqr(values)",
            args: &IQR_ARGS,
            returns: "Number",
            examples: &IQR_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &IQR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "iqr") {
            return Value::Error(e);
        }

        let p25 = Number::from_i64(25);
        let p75 = Number::from_i64(75);

        let q1 = match percentile_impl(&numbers, &p25) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let q3 = match percentile_impl(&numbers, &p75) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        Value::Number(q3.sub(&q1))
    }
}

// ============ MAD (Median Absolute Deviation) ============

pub struct Mad;

static MAD_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static MAD_EXAMPLES: [&str; 1] = ["mad(1, 1, 2, 2, 4, 6, 9) → 1"];

static MAD_RELATED: [&str; 2] = ["median", "stddev"];

impl FunctionPlugin for Mad {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mad",
            description: "Median absolute deviation",
            usage: "mad(values)",
            args: &MAD_ARGS,
            returns: "Number",
            examples: &MAD_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &MAD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "mad") {
            return Value::Error(e);
        }

        // Calculate median
        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();
        let median = if n % 2 == 1 {
            sorted_nums[n / 2].clone()
        } else {
            let mid1 = &sorted_nums[n / 2 - 1];
            let mid2 = &sorted_nums[n / 2];
            let two = Number::from_i64(2);
            mid1.add(mid2).checked_div(&two).unwrap_or(mid1.clone())
        };

        // Calculate absolute deviations from median
        let abs_devs: Vec<Number> = numbers
            .iter()
            .map(|x| x.sub(&median).abs())
            .collect();

        // Return median of absolute deviations
        let sorted_devs = sorted(&abs_devs);
        let m = sorted_devs.len();
        if m % 2 == 1 {
            Value::Number(sorted_devs[m / 2].clone())
        } else {
            let mid1 = &sorted_devs[m / 2 - 1];
            let mid2 = &sorted_devs[m / 2];
            let two = Number::from_i64(2);
            match mid1.add(mid2).checked_div(&two) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            }
        }
    }
}

// ============ CV (Coefficient of Variation) ============

pub struct Cv;

static CV_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static CV_EXAMPLES: [&str; 1] = ["cv(10, 20, 30) → 0.5"];

static CV_RELATED: [&str; 2] = ["stddev", "mean"];

impl FunctionPlugin for Cv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cv",
            description: "Coefficient of variation: stddev/mean",
            usage: "cv(values)",
            args: &CV_ARGS,
            returns: "Number",
            examples: &CV_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &CV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "cv") {
            return Value::Error(e);
        }

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if m.is_zero() {
            return Value::Error(FolioError::domain_error(
                "cv() undefined when mean is zero",
            ));
        }

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        match sd.checked_div(&m) {
            Ok(result) => Value::Number(result.abs()),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ SE (Standard Error) ============

pub struct Se;

static SE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Sample data",
    optional: false,
    default: None,
}];

static SE_EXAMPLES: [&str; 1] = ["se(1, 2, 3, 4, 5) → 0.707..."];

static SE_RELATED: [&str; 2] = ["stddev", "ci"];

impl FunctionPlugin for Se {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "se",
            description: "Standard error: stddev/√n",
            usage: "se(values)",
            args: &SE_ARGS,
            returns: "Number",
            examples: &SE_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &SE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "se") {
            return Value::Error(e);
        }

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let n = Number::from_i64(numbers.len() as i64);
        let sqrt_n = match n.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        match sd.checked_div(&sqrt_n) {
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
    fn test_variance() {
        let variance = Variance;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(7)),
            Value::Number(Number::from_i64(9)),
        ])];
        let result = variance.call(&args, &eval_ctx());
        // Sample variance should be approximately 4.571
        let num = result.as_number().unwrap();
        let decimal = num.as_decimal(2);
        assert!(decimal.starts_with("4.5"));
    }

    #[test]
    fn test_range() {
        let range = Range;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(10)),
        ])];
        let result = range.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(9));
    }
}
