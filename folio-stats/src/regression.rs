//! Regression functions: linear_reg, slope, intercept, r_squared, predict, residuals

use folio_plugin::prelude::*;
use crate::helpers::{extract_two_lists, mean, variance_impl};
use std::collections::HashMap;

/// Calculate linear regression coefficients
fn linear_regression_impl(x: &[Number], y: &[Number], precision: u32) -> Result<(Number, Number, Number, Number), FolioError> {
    let n = x.len();
    if n < 2 {
        return Err(FolioError::domain_error(
            "Linear regression requires at least 2 points",
        ));
    }

    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    // Calculate sums for slope
    let mut sum_xy_dev = Number::from_i64(0);
    let mut sum_xx_dev = Number::from_i64(0);

    for (xi, yi) in x.iter().zip(y.iter()) {
        let dev_x = xi.sub(&mean_x);
        let dev_y = yi.sub(&mean_y);
        sum_xy_dev = sum_xy_dev.add(&dev_x.mul(&dev_y));
        sum_xx_dev = sum_xx_dev.add(&dev_x.mul(&dev_x));
    }

    if sum_xx_dev.is_zero() {
        return Err(FolioError::domain_error(
            "Cannot perform regression: x has zero variance",
        ));
    }

    // slope = Σ(x-x̄)(y-ȳ) / Σ(x-x̄)²
    let slope = sum_xy_dev.checked_div(&sum_xx_dev)?;

    // intercept = ȳ - slope * x̄
    let intercept = mean_y.sub(&slope.mul(&mean_x));

    // Calculate R²
    let var_y = variance_impl(y, false)?;
    if var_y.is_zero() {
        // Perfect prediction if y has no variance
        return Ok((slope, intercept, Number::from_i64(1), Number::from_i64(1)));
    }

    // SS_res = Σ(y - ŷ)²
    let mut ss_res = Number::from_i64(0);
    for (xi, yi) in x.iter().zip(y.iter()) {
        let y_pred = intercept.add(&slope.mul(xi));
        let residual = yi.sub(&y_pred);
        ss_res = ss_res.add(&residual.mul(&residual));
    }

    // SS_tot = Σ(y - ȳ)²
    let mut ss_tot = Number::from_i64(0);
    for yi in y {
        let dev = yi.sub(&mean_y);
        ss_tot = ss_tot.add(&dev.mul(&dev));
    }

    // R² = 1 - SS_res/SS_tot
    let r_squared = if ss_tot.is_zero() {
        Number::from_i64(1)
    } else {
        let ratio = ss_res.checked_div(&ss_tot)?;
        Number::from_i64(1).sub(&ratio)
    };

    // r = sign(slope) * sqrt(R²)
    let r = match r_squared.sqrt(precision) {
        Ok(sqrt_r2) => {
            if slope.is_negative() {
                Number::from_i64(0).sub(&sqrt_r2)
            } else {
                sqrt_r2
            }
        }
        Err(_) => Number::from_i64(0),
    };

    Ok((slope, intercept, r_squared, r))
}

// ============ LinearReg ============

pub struct LinearReg;

static LINEAR_REG_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static LINEAR_REG_EXAMPLES: [&str; 1] = ["linear_reg([1,2,3], [2,4,6]) → {slope: 2, intercept: 0, ...}"];

static LINEAR_REG_RELATED: [&str; 3] = ["slope", "intercept", "r_squared"];

impl FunctionPlugin for LinearReg {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "linear_reg",
            description: "Full linear regression result",
            usage: "linear_reg(x, y)",
            args: &LINEAR_REG_ARGS,
            returns: "Object",
            examples: &LINEAR_REG_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &LINEAR_REG_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let (slope, intercept, r_squared, r) = match linear_regression_impl(&x, &y, ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        // Calculate standard error
        let n = x.len();
        let mut ss_res = Number::from_i64(0);
        for (xi, yi) in x.iter().zip(y.iter()) {
            let y_pred = intercept.add(&slope.mul(xi));
            let residual = yi.sub(&y_pred);
            ss_res = ss_res.add(&residual.mul(&residual));
        }

        let std_error = if n > 2 {
            let df = Number::from_i64((n - 2) as i64);
            match ss_res.checked_div(&df) {
                Ok(mse) => mse.sqrt(ctx.precision).unwrap_or(Number::from_i64(0)),
                Err(_) => Number::from_i64(0),
            }
        } else {
            Number::from_i64(0)
        };

        let mut result = HashMap::new();
        result.insert("slope".to_string(), Value::Number(slope));
        result.insert("intercept".to_string(), Value::Number(intercept));
        result.insert("r_squared".to_string(), Value::Number(r_squared));
        result.insert("r".to_string(), Value::Number(r));
        result.insert("std_error".to_string(), Value::Number(std_error));
        result.insert("n".to_string(), Value::Number(Number::from_i64(n as i64)));

        Value::Object(result)
    }
}

// ============ Slope ============

pub struct Slope;

static SLOPE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static SLOPE_EXAMPLES: [&str; 1] = ["slope([1,2,3], [2,4,6]) → 2"];

static SLOPE_RELATED: [&str; 2] = ["intercept", "linear_reg"];

impl FunctionPlugin for Slope {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "slope",
            description: "Slope of linear regression",
            usage: "slope(x, y)",
            args: &SLOPE_ARGS,
            returns: "Number",
            examples: &SLOPE_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &SLOPE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match linear_regression_impl(&x, &y, ctx.precision) {
            Ok((slope, _, _, _)) => Value::Number(slope),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Intercept ============

pub struct Intercept;

static INTERCEPT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static INTERCEPT_EXAMPLES: [&str; 1] = ["intercept([1,2,3], [3,5,7]) → 1"];

static INTERCEPT_RELATED: [&str; 2] = ["slope", "linear_reg"];

impl FunctionPlugin for Intercept {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "intercept",
            description: "Y-intercept of linear regression",
            usage: "intercept(x, y)",
            args: &INTERCEPT_ARGS,
            returns: "Number",
            examples: &INTERCEPT_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &INTERCEPT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match linear_regression_impl(&x, &y, ctx.precision) {
            Ok((_, intercept, _, _)) => Value::Number(intercept),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ RSquared ============

pub struct RSquared;

static R_SQUARED_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static R_SQUARED_EXAMPLES: [&str; 1] = ["r_squared([1,2,3], [2,4,6]) → 1"];

static R_SQUARED_RELATED: [&str; 2] = ["correlation", "linear_reg"];

impl FunctionPlugin for RSquared {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "r_squared",
            description: "Coefficient of determination (R²)",
            usage: "r_squared(x, y)",
            args: &R_SQUARED_ARGS,
            returns: "Number",
            examples: &R_SQUARED_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &R_SQUARED_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match linear_regression_impl(&x, &y, ctx.precision) {
            Ok((_, _, r_squared, _)) => Value::Number(r_squared),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Predict ============

pub struct Predict;

static PREDICT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "reg",
        typ: "Object",
        description: "Regression result from linear_reg()",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "X value to predict",
        optional: false,
        default: None,
    },
];

static PREDICT_EXAMPLES: [&str; 1] = ["predict(linear_reg([1,2,3], [2,4,6]), 4) → 8"];

static PREDICT_RELATED: [&str; 2] = ["linear_reg", "residuals"];

impl FunctionPlugin for Predict {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "predict",
            description: "Predict y from regression object",
            usage: "predict(reg, x)",
            args: &PREDICT_ARGS,
            returns: "Number",
            examples: &PREDICT_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &PREDICT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("predict", 2, args.len()));
        }

        let reg = match &args[0] {
            Value::Object(obj) => obj,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("predict", "reg", "Object", other.type_name())),
        };

        let x = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("predict", "x", "Number", other.type_name())),
        };

        let slope = match reg.get("slope") {
            Some(Value::Number(n)) => n,
            _ => return Value::Error(FolioError::undefined_field("slope")),
        };

        let intercept = match reg.get("intercept") {
            Some(Value::Number(n)) => n,
            _ => return Value::Error(FolioError::undefined_field("intercept")),
        };

        // y = intercept + slope * x
        Value::Number(intercept.add(&slope.mul(x)))
    }
}

// ============ Residuals ============

pub struct Residuals;

static RESIDUALS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static RESIDUALS_EXAMPLES: [&str; 1] = ["residuals([1,2,3], [2.1, 3.9, 6.1])"];

static RESIDUALS_RELATED: [&str; 2] = ["linear_reg", "predict"];

impl FunctionPlugin for Residuals {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "residuals",
            description: "List of (y - ŷ) residuals",
            usage: "residuals(x, y)",
            args: &RESIDUALS_ARGS,
            returns: "List<Number>",
            examples: &RESIDUALS_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &RESIDUALS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let (slope, intercept, _, _) = match linear_regression_impl(&x, &y, ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let residuals: Vec<Value> = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| {
                let y_pred = intercept.add(&slope.mul(xi));
                Value::Number(yi.sub(&y_pred))
            })
            .collect();

        Value::List(residuals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_slope() {
        let slope = Slope;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(6)),
            ]),
        ];
        let result = slope.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_intercept() {
        let intercept = Intercept;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(5)),
                Value::Number(Number::from_i64(7)),
            ]),
        ];
        let result = intercept.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }

    #[test]
    fn test_r_squared_perfect() {
        let r_squared = RSquared;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(6)),
            ]),
        ];
        let result = r_squared.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }
}
