//! Shape functions: skewness, kurtosis, count, product

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_min_count, mean, variance_impl};

// ============ Skewness ============

pub struct Skewness;

static SKEWNESS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static SKEWNESS_EXAMPLES: [&str; 1] = ["skewness([1,2,2,3,3,3,4,4,5]) → positive"];

static SKEWNESS_RELATED: [&str; 2] = ["kurtosis", "stddev"];

impl FunctionPlugin for Skewness {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "skewness",
            description: "Measure of asymmetry (Fisher's skewness)",
            usage: "skewness(values)",
            args: &SKEWNESS_ARGS,
            returns: "Number",
            examples: &SKEWNESS_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &SKEWNESS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "skewness") {
            return Value::Error(e);
        }

        let n = numbers.len();
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
                "skewness() undefined when stddev is zero",
            ));
        }

        // Calculate sum of cubed deviations
        let mut sum_cubed = Number::from_i64(0);
        for x in &numbers {
            let dev = x.sub(&m);
            let cubed = dev.mul(&dev).mul(&dev);
            sum_cubed = sum_cubed.add(&cubed);
        }

        // Fisher's skewness: n / ((n-1)(n-2)) * sum((x-mean)^3) / sd^3
        let n_num = Number::from_i64(n as i64);
        let n_minus_1 = Number::from_i64((n - 1) as i64);
        let n_minus_2 = Number::from_i64((n - 2) as i64);

        let sd_cubed = sd.mul(&sd).mul(&sd);
        let adjustment = match n_num.checked_div(&n_minus_1.mul(&n_minus_2)) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let m3 = match sum_cubed.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        match m3.checked_div(&sd_cubed) {
            Ok(raw) => Value::Number(adjustment.mul(&n_num).mul(&raw)),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Kurtosis ============

pub struct Kurtosis;

static KURTOSIS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static KURTOSIS_EXAMPLES: [&str; 1] = ["kurtosis([1,2,3,4,5]) → -1.3"];

static KURTOSIS_RELATED: [&str; 2] = ["skewness", "stddev"];

impl FunctionPlugin for Kurtosis {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "kurtosis",
            description: "Excess kurtosis (Fisher's definition, normal = 0)",
            usage: "kurtosis(values)",
            args: &KURTOSIS_ARGS,
            returns: "Number",
            examples: &KURTOSIS_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &KURTOSIS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 4, "kurtosis") {
            return Value::Error(e);
        }

        let n = numbers.len();
        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if var.is_zero() {
            return Value::Error(FolioError::domain_error(
                "kurtosis() undefined when variance is zero",
            ));
        }

        // Calculate sum of fourth power deviations
        let mut sum_fourth = Number::from_i64(0);
        for x in &numbers {
            let dev = x.sub(&m);
            let fourth = dev.mul(&dev).mul(&dev).mul(&dev);
            sum_fourth = sum_fourth.add(&fourth);
        }

        // Excess kurtosis formula
        let n_num = Number::from_i64(n as i64);
        let n_minus_1 = Number::from_i64((n - 1) as i64);
        let n_minus_2 = Number::from_i64((n - 2) as i64);
        let n_minus_3 = Number::from_i64((n - 3) as i64);
        let three = Number::from_i64(3);

        let var_squared = var.mul(&var);

        // m4 = sum_fourth / n
        let m4 = match sum_fourth.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Raw kurtosis = m4 / var^2
        let raw_kurtosis = match m4.checked_div(&var_squared) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Fisher's excess kurtosis adjustment
        // ((n+1)*n / ((n-1)(n-2)(n-3))) * raw - 3*(n-1)^2 / ((n-2)(n-3))
        let n_plus_1 = Number::from_i64((n + 1) as i64);

        let coef1_num = n_plus_1.mul(&n_num);
        let coef1_den = n_minus_1.mul(&n_minus_2).mul(&n_minus_3);
        let coef1 = match coef1_num.checked_div(&coef1_den) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let coef2_num = three.mul(&n_minus_1).mul(&n_minus_1);
        let coef2_den = n_minus_2.mul(&n_minus_3);
        let coef2 = match coef2_num.checked_div(&coef2_den) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        Value::Number(coef1.mul(&raw_kurtosis).sub(&coef2))
    }
}

// ============ Count ============

pub struct Count;

static COUNT_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static COUNT_EXAMPLES: [&str; 1] = ["count([1,2,3,4,5]) → 5"];

static COUNT_RELATED: [&str; 1] = ["sum"];

impl FunctionPlugin for Count {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "count",
            description: "Number of elements",
            usage: "count(values)",
            args: &COUNT_ARGS,
            returns: "Number",
            examples: &COUNT_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &COUNT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        Value::Number(Number::from_i64(numbers.len() as i64))
    }
}

// ============ Product ============

pub struct Product;

static PRODUCT_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to multiply",
    optional: false,
    default: None,
}];

static PRODUCT_EXAMPLES: [&str; 1] = ["product([1,2,3,4,5]) → 120"];

static PRODUCT_RELATED: [&str; 2] = ["sum", "gmean"];

impl FunctionPlugin for Product {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "product",
            description: "Product of all elements",
            usage: "product(values)",
            args: &PRODUCT_ARGS,
            returns: "Number",
            examples: &PRODUCT_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &PRODUCT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if numbers.is_empty() {
            return Value::Number(Number::from_i64(1)); // Empty product is 1
        }

        let result = numbers
            .iter()
            .fold(Number::from_i64(1), |acc, n| acc.mul(n));

        Value::Number(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_count() {
        let count = Count;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = count.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_product() {
        let product = Product;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = product.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(120));
    }

    #[test]
    fn test_count_empty() {
        let count = Count;
        let args = vec![Value::List(vec![])];
        let result = count.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(0));
    }
}
