//! Bivariate functions: covariance, correlation, spearman

use folio_plugin::prelude::*;
use crate::helpers::{extract_two_lists, mean, variance_impl, ranks};

// ============ Covariance (Sample) ============

pub struct Covariance;

static COVARIANCE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static COVARIANCE_EXAMPLES: [&str; 1] = ["covariance([1,2,3], [4,5,6]) → 1"];

static COVARIANCE_RELATED: [&str; 2] = ["covariance_p", "correlation"];

impl FunctionPlugin for Covariance {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "covariance",
            description: "Sample covariance (divides by n-1)",
            usage: "covariance(x, y)",
            args: &COVARIANCE_ARGS,
            returns: "Number",
            examples: &COVARIANCE_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &COVARIANCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match covariance_impl(&x, &y, true) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Covariance (Population) ============

pub struct CovarianceP;

static COVARIANCE_P_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static COVARIANCE_P_EXAMPLES: [&str; 1] = ["covariance_p([1,2,3], [4,5,6]) → 0.667"];

static COVARIANCE_P_RELATED: [&str; 2] = ["covariance", "correlation"];

impl FunctionPlugin for CovarianceP {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "covariance_p",
            description: "Population covariance (divides by n)",
            usage: "covariance_p(x, y)",
            args: &COVARIANCE_P_ARGS,
            returns: "Number",
            examples: &COVARIANCE_P_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &COVARIANCE_P_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match covariance_impl(&x, &y, false) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

/// Calculate covariance (sample or population)
fn covariance_impl(x: &[Number], y: &[Number], sample: bool) -> Result<Number, FolioError> {
    let n = x.len();
    if n == 0 {
        return Err(FolioError::domain_error("Cannot calculate covariance of empty lists"));
    }
    if sample && n < 2 {
        return Err(FolioError::domain_error(
            "Sample covariance requires at least 2 values",
        ));
    }

    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    let mut sum_products = Number::from_i64(0);
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dev_x = xi.sub(&mean_x);
        let dev_y = yi.sub(&mean_y);
        sum_products = sum_products.add(&dev_x.mul(&dev_y));
    }

    let divisor = if sample {
        Number::from_i64((n - 1) as i64)
    } else {
        Number::from_i64(n as i64)
    };

    sum_products.checked_div(&divisor).map_err(|e| e.into())
}

// ============ Correlation (Pearson) ============

pub struct Correlation;

static CORRELATION_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static CORRELATION_EXAMPLES: [&str; 1] = ["correlation([1,2,3], [1,2,3]) → 1"];

static CORRELATION_RELATED: [&str; 2] = ["covariance", "spearman"];

impl FunctionPlugin for Correlation {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "correlation",
            description: "Pearson correlation coefficient r",
            usage: "correlation(x, y)",
            args: &CORRELATION_ARGS,
            returns: "Number",
            examples: &CORRELATION_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &CORRELATION_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "correlation() requires at least 2 pairs",
            ));
        }

        // r = cov(x,y) / (sd_x * sd_y)
        let cov = match covariance_impl(&x, &y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_x = match variance_impl(&x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_y = match variance_impl(&y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd_x = match var_x.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let sd_y = match var_y.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd_x.is_zero() || sd_y.is_zero() {
            return Value::Error(FolioError::domain_error(
                "correlation() undefined when a variable has zero variance",
            ));
        }

        match cov.checked_div(&sd_x.mul(&sd_y)) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Spearman Rank Correlation ============

pub struct Spearman;

static SPEARMAN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static SPEARMAN_EXAMPLES: [&str; 1] = ["spearman([1,2,3], [1,2,3]) → 1"];

static SPEARMAN_RELATED: [&str; 2] = ["correlation", "rank"];

impl FunctionPlugin for Spearman {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "spearman",
            description: "Spearman rank correlation coefficient",
            usage: "spearman(x, y)",
            args: &SPEARMAN_ARGS,
            returns: "Number",
            examples: &SPEARMAN_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &SPEARMAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "spearman() requires at least 2 pairs",
            ));
        }

        // Convert to ranks and compute Pearson correlation on ranks
        let ranks_x = ranks(&x);
        let ranks_y = ranks(&y);

        // r = cov(rank_x, rank_y) / (sd_rank_x * sd_rank_y)
        let cov = match covariance_impl(&ranks_x, &ranks_y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_x = match variance_impl(&ranks_x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_y = match variance_impl(&ranks_y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd_x = match var_x.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let sd_y = match var_y.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd_x.is_zero() || sd_y.is_zero() {
            return Value::Error(FolioError::domain_error(
                "spearman() undefined when a variable has zero variance",
            ));
        }

        match cov.checked_div(&sd_x.mul(&sd_y)) {
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
    fn test_correlation_perfect() {
        let correlation = Correlation;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
        ];
        let result = correlation.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        // Should be exactly 1
        assert_eq!(num.to_i64(), Some(1));
    }

    #[test]
    fn test_covariance() {
        let covariance = Covariance;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(5)),
                Value::Number(Number::from_i64(6)),
            ]),
        ];
        let result = covariance.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }
}
