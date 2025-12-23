//! Central tendency functions: mean, median, mode, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, mean, sorted};
use std::collections::HashMap;

// ============ Mean ============

pub struct Mean;

static MEAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to average",
    optional: false,
    default: None,
}];

static MEAN_EXAMPLES: [&str; 3] = [
    "mean(1, 2, 3, 4, 5) → 3",
    "mean(Data.Value) → average of column",
    "mean([10, 20, 30]) → 20",
];

static MEAN_RELATED: [&str; 4] = ["median", "mode", "gmean", "hmean"];

impl FunctionPlugin for Mean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mean",
            description: "Arithmetic mean (average) of values",
            usage: "mean(values) or mean(a, b, c, ...)",
            args: &MEAN_ARGS,
            returns: "Number",
            examples: &MEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &MEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "mean") {
            return Value::Error(e);
        }

        match mean(&numbers) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Median ============

pub struct Median;

static MEDIAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to find median of",
    optional: false,
    default: None,
}];

static MEDIAN_EXAMPLES: [&str; 2] = [
    "median(1, 2, 3, 4, 5) → 3",
    "median(1, 2, 3, 4) → 2.5",
];

static MEDIAN_RELATED: [&str; 3] = ["mean", "mode", "percentile"];

impl FunctionPlugin for Median {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "median",
            description: "Middle value (average of two middle if even count)",
            usage: "median(values)",
            args: &MEDIAN_ARGS,
            returns: "Number",
            examples: &MEDIAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &MEDIAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "median") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();

        if n % 2 == 1 {
            Value::Number(sorted_nums[n / 2].clone())
        } else {
            let mid1 = &sorted_nums[n / 2 - 1];
            let mid2 = &sorted_nums[n / 2];
            let two = Number::from_i64(2);
            match mid1.add(mid2).checked_div(&two) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            }
        }
    }
}

// ============ Mode ============

pub struct Mode;

static MODE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to find mode of",
    optional: false,
    default: None,
}];

static MODE_EXAMPLES: [&str; 2] = [
    "mode(1, 2, 2, 3) → 2",
    "mode(1, 1, 2, 2) → [1, 2]",
];

static MODE_RELATED: [&str; 2] = ["mean", "median"];

impl FunctionPlugin for Mode {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mode",
            description: "Most frequent value. Returns List if tie",
            usage: "mode(values)",
            args: &MODE_ARGS,
            returns: "Number | List<Number>",
            examples: &MODE_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &MODE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "mode") {
            return Value::Error(e);
        }

        // Count frequencies using string representation as key
        let mut counts: HashMap<String, (Number, usize)> = HashMap::new();
        for n in &numbers {
            let key = n.as_decimal(20);
            let entry = counts.entry(key).or_insert_with(|| (n.clone(), 0));
            entry.1 += 1;
        }

        let max_count = counts.values().map(|(_, c)| *c).max().unwrap_or(0);
        let modes: Vec<Number> = counts
            .into_iter()
            .filter(|(_, (_, c))| *c == max_count)
            .map(|(_, (n, _))| n)
            .collect();

        if modes.len() == 1 {
            Value::Number(modes.into_iter().next().unwrap())
        } else {
            Value::List(modes.into_iter().map(Value::Number).collect())
        }
    }
}

// ============ Geometric Mean ============

pub struct GeometricMean;

static GMEAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Positive numbers",
    optional: false,
    default: None,
}];

static GMEAN_EXAMPLES: [&str; 1] = ["gmean(1, 2, 4, 8) → 2.828..."];

static GMEAN_RELATED: [&str; 2] = ["mean", "hmean"];

impl FunctionPlugin for GeometricMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "gmean",
            description: "Geometric mean: (∏x)^(1/n). Error if any x ≤ 0",
            usage: "gmean(values)",
            args: &GMEAN_ARGS,
            returns: "Number",
            examples: &GMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &GMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "gmean") {
            return Value::Error(e);
        }

        // Check all values are positive
        let zero = Number::from_i64(0);
        for n in &numbers {
            if n.sub(&zero).is_negative() || n.is_zero() {
                return Value::Error(FolioError::domain_error(
                    "gmean() requires all positive values",
                ));
            }
        }

        // Calculate: exp(mean(ln(x_i)))
        let mut sum_ln = Number::from_i64(0);
        for n in &numbers {
            match n.ln(ctx.precision) {
                Ok(ln_val) => sum_ln = sum_ln.add(&ln_val),
                Err(e) => return Value::Error(e.into()),
            }
        }

        let count = Number::from_i64(numbers.len() as i64);
        match sum_ln.checked_div(&count) {
            Ok(mean_ln) => Value::Number(mean_ln.exp(ctx.precision)),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Harmonic Mean ============

pub struct HarmonicMean;

static HMEAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Non-zero numbers",
    optional: false,
    default: None,
}];

static HMEAN_EXAMPLES: [&str; 1] = ["hmean(1, 2, 4) → 1.714..."];

static HMEAN_RELATED: [&str; 2] = ["mean", "gmean"];

impl FunctionPlugin for HarmonicMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hmean",
            description: "Harmonic mean: n/Σ(1/x). Error if any x = 0",
            usage: "hmean(values)",
            args: &HMEAN_ARGS,
            returns: "Number",
            examples: &HMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &HMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "hmean") {
            return Value::Error(e);
        }

        // Check all values are non-zero
        for n in &numbers {
            if n.is_zero() {
                return Value::Error(FolioError::domain_error(
                    "hmean() requires all non-zero values",
                ));
            }
        }

        // Calculate: n / Σ(1/x_i)
        let one = Number::from_i64(1);
        let mut sum_reciprocals = Number::from_i64(0);
        for n in &numbers {
            match one.checked_div(n) {
                Ok(recip) => sum_reciprocals = sum_reciprocals.add(&recip),
                Err(e) => return Value::Error(e.into()),
            }
        }

        let count = Number::from_i64(numbers.len() as i64);
        match count.checked_div(&sum_reciprocals) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Trimmed Mean ============

pub struct TrimmedMean;

static TMEAN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number> | Number...",
        description: "Numbers to average",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pct",
        typ: "Number",
        description: "Percentage to trim from each tail (0-50)",
        optional: false,
        default: None,
    },
];

static TMEAN_EXAMPLES: [&str; 1] = ["tmean([1,2,3,4,100], 20) → 3 (trims extremes)"];

static TMEAN_RELATED: [&str; 2] = ["mean", "median"];

impl FunctionPlugin for TrimmedMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tmean",
            description: "Trimmed mean excluding pct% from each tail",
            usage: "tmean(values, pct)",
            args: &TMEAN_ARGS,
            returns: "Number",
            examples: &TMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &TMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("tmean", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let pct = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("tmean", "pct", "Number", other.type_name())),
        };

        if let Err(e) = require_non_empty(&numbers, "tmean") {
            return Value::Error(e);
        }

        // Validate pct is in [0, 50)
        let pct_f64 = pct.to_f64().unwrap_or(0.0);
        if pct_f64 < 0.0 || pct_f64 >= 50.0 {
            return Value::Error(FolioError::domain_error(
                "tmean() requires 0 <= pct < 50",
            ));
        }

        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();

        // Calculate how many to trim from each end
        let trim_count = ((n as f64) * pct_f64 / 100.0).floor() as usize;

        if 2 * trim_count >= n {
            return Value::Error(FolioError::domain_error(
                "tmean() would trim all values",
            ));
        }

        let trimmed = &sorted_nums[trim_count..(n - trim_count)];
        match mean(trimmed) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Weighted Mean ============

pub struct WeightedMean;

static WMEAN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Values to average",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "weights",
        typ: "List<Number>",
        description: "Weights (must be same length as values)",
        optional: false,
        default: None,
    },
];

static WMEAN_EXAMPLES: [&str; 1] = ["wmean([80, 90, 100], [1, 2, 1]) → 90"];

static WMEAN_RELATED: [&str; 1] = ["mean"];

impl FunctionPlugin for WeightedMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "wmean",
            description: "Weighted mean: Σ(w·x)/Σw",
            usage: "wmean(values, weights)",
            args: &WMEAN_ARGS,
            returns: "Number",
            examples: &WMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &WMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("wmean", 2, args.len()));
        }

        let values = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let weights = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if values.len() != weights.len() {
            return Value::Error(FolioError::domain_error(format!(
                "wmean() requires equal length lists: {} values vs {} weights",
                values.len(),
                weights.len()
            )));
        }

        if let Err(e) = require_non_empty(&values, "wmean") {
            return Value::Error(e);
        }

        // Check all weights are non-negative
        for w in &weights {
            if w.is_negative() {
                return Value::Error(FolioError::domain_error(
                    "wmean() requires non-negative weights",
                ));
            }
        }

        let mut weighted_sum = Number::from_i64(0);
        let mut weight_sum = Number::from_i64(0);

        for (v, w) in values.iter().zip(weights.iter()) {
            weighted_sum = weighted_sum.add(&v.mul(w));
            weight_sum = weight_sum.add(w);
        }

        if weight_sum.is_zero() {
            return Value::Error(FolioError::domain_error(
                "wmean() requires at least one non-zero weight",
            ));
        }

        match weighted_sum.checked_div(&weight_sum) {
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
    fn test_mean() {
        let mean = Mean;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = mean.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_median_odd() {
        let median = Median;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(2)),
        ])];
        let result = median.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_median_even() {
        let median = Median;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ])];
        let result = median.call(&args, &eval_ctx());
        // Median of [1,2,3,4] = (2+3)/2 = 2.5
        let num = result.as_number().unwrap();
        let decimal = num.as_decimal(1);
        assert!(decimal.starts_with("2.5"));
    }

    #[test]
    fn test_mode_single() {
        let mode = Mode;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = mode.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }
}
