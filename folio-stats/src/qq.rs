//! Q-Q (Quantile-Quantile) analysis functions

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted, mean, variance_impl};

// ============================================================================
// QQPoints - Generate Q-Q plot points
// ============================================================================

pub struct QQPoints;

static QQ_POINTS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "distribution",
        typ: "Text",
        description: "Distribution: 'normal' (default)",
        optional: true,
        default: Some("normal"),
    },
];
static QQ_POINTS_EXAMPLES: [&str; 1] = ["qq_points(data) → {theoretical: [...], sample: [...], r_squared: 0.98}"];
static QQ_POINTS_RELATED: [&str; 2] = ["qq_residuals", "is_normal"];

impl FunctionPlugin for QQPoints {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "qq_points",
            description: "Generate Q-Q plot points for visual normality assessment",
            usage: "qq_points(list, distribution?)",
            args: &QQ_POINTS_ARGS,
            returns: "Object",
            examples: &QQ_POINTS_EXAMPLES,
            category: "distribution",
            source: None,
            related: &QQ_POINTS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("qq_points", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "qq_points") {
            return Value::Error(e);
        }

        // Get distribution (default "normal")
        let dist = if args.len() == 2 {
            match &args[1] {
                Value::Text(s) => s.to_lowercase(),
                other => return Value::Error(FolioError::arg_type("qq_points", "distribution", "Text", other.type_name())),
            }
        } else {
            "normal".to_string()
        };

        if dist != "normal" {
            return Value::Error(FolioError::domain_error(format!(
                "Only 'normal' distribution currently supported, got '{}'",
                dist
            )));
        }

        let n = numbers.len();
        let sorted_data = sorted(&numbers);

        // Calculate theoretical quantiles
        let theoretical: Vec<Number> = (0..n).map(|i| {
            // Filliben approximation for plotting positions
            let p = if i == 0 {
                1.0 - 0.5_f64.powf(1.0 / n as f64)
            } else if i == n - 1 {
                0.5_f64.powf(1.0 / n as f64)
            } else {
                (i as f64 + 1.0 - 0.3175) / (n as f64 + 0.365)
            };
            let q = normal_quantile(p);
            Number::from_str(&format!("{:.10}", q)).unwrap_or(Number::from_i64(0))
        }).collect();

        // Sample quantiles (sorted data)
        let sample: Vec<Number> = sorted_data.clone();

        // Calculate R-squared for linearity assessment
        let m_x = match mean(&theoretical) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };
        let m_y = match mean(&sample) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Calculate regression coefficients and R²
        let mut ss_xy = Number::from_i64(0);
        let mut ss_xx = Number::from_i64(0);
        let mut ss_yy = Number::from_i64(0);

        for i in 0..n {
            let dx = theoretical[i].sub(&m_x);
            let dy = sample[i].sub(&m_y);
            ss_xy = ss_xy.add(&dx.mul(&dy));
            ss_xx = ss_xx.add(&dx.mul(&dx));
            ss_yy = ss_yy.add(&dy.mul(&dy));
        }

        // Slope = ss_xy / ss_xx (should be ≈ σ)
        let slope = if ss_xx.is_zero() {
            Number::from_i64(0)
        } else {
            match ss_xy.checked_div(&ss_xx) {
                Ok(s) => s,
                Err(_) => Number::from_i64(0),
            }
        };

        // Intercept = mean_y - slope * mean_x (should be ≈ μ)
        let intercept = m_y.sub(&slope.mul(&m_x));

        // R² = (ss_xy)² / (ss_xx * ss_yy)
        let ss_prod = ss_xx.mul(&ss_yy);
        let r_squared = if ss_prod.is_zero() {
            Number::from_i64(1) // Perfect fit if no variation
        } else {
            let ss_xy_sq = ss_xy.mul(&ss_xy);
            match ss_xy_sq.checked_div(&ss_prod) {
                Ok(r2) => r2,
                Err(_) => Number::from_i64(0),
            }
        };

        let theoretical_values: Vec<Value> = theoretical.into_iter().map(Value::Number).collect();
        let sample_values: Vec<Value> = sample.into_iter().map(Value::Number).collect();

        let mut result = HashMap::new();
        result.insert("theoretical".to_string(), Value::List(theoretical_values));
        result.insert("sample".to_string(), Value::List(sample_values));
        result.insert("r_squared".to_string(), Value::Number(r_squared));
        result.insert("slope".to_string(), Value::Number(slope));
        result.insert("intercept".to_string(), Value::Number(intercept));

        Value::Object(result)
    }
}

// ============================================================================
// QQResiduals - Q-Q residuals from normal distribution
// ============================================================================

pub struct QQResiduals;

static QQ_RESIDUALS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
];
static QQ_RESIDUALS_EXAMPLES: [&str; 1] = ["qq_residuals(data) → [0.1, -0.05, 0.02, ...]"];
static QQ_RESIDUALS_RELATED: [&str; 2] = ["qq_points", "residuals"];

impl FunctionPlugin for QQResiduals {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "qq_residuals",
            description: "Deviations from theoretical normal quantiles",
            usage: "qq_residuals(list)",
            args: &QQ_RESIDUALS_ARGS,
            returns: "List",
            examples: &QQ_RESIDUALS_EXAMPLES,
            category: "distribution",
            source: None,
            related: &QQ_RESIDUALS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("qq_residuals", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "qq_residuals") {
            return Value::Error(e);
        }

        let n = numbers.len();

        // Calculate sample mean and stddev
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };
        let variance = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let stddev = match variance.sqrt(50) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };

        if stddev.is_zero() {
            // All values are the same
            return Value::List(vec![Value::Number(Number::from_i64(0)); n]);
        }

        let sorted_data = sorted(&numbers);

        // Calculate residuals: observed - expected (under normality)
        let residuals: Vec<Value> = (0..n).map(|i| {
            // Expected value at this position for a normal distribution
            let p = if i == 0 {
                1.0 - 0.5_f64.powf(1.0 / n as f64)
            } else if i == n - 1 {
                0.5_f64.powf(1.0 / n as f64)
            } else {
                (i as f64 + 1.0 - 0.3175) / (n as f64 + 0.365)
            };
            let z = normal_quantile(p);
            let expected = m.add(&stddev.mul(&Number::from_str(&format!("{:.10}", z)).unwrap_or(Number::from_i64(0))));

            // Residual = observed - expected
            let residual = sorted_data[i].sub(&expected);
            Value::Number(residual)
        }).collect();

        Value::List(residuals)
    }
}

// ============================================================================
// Helper: Standard normal quantile function
// ============================================================================

/// Standard normal quantile (inverse CDF) - Rational approximation
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < 1e-10 {
        return 0.0;
    }

    let a = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239e0,
    ];
    let b = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    let c = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838e0,
        -2.549732539343734e0,
        4.374664141464968e0,
        2.938163982698783e0,
    ];
    let d = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996e0,
        3.754408661907416e0,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    fn make_list(values: &[i64]) -> Value {
        Value::List(values.iter().map(|v| Value::Number(Number::from_i64(*v))).collect())
    }

    #[test]
    fn test_qq_points() {
        let qq = QQPoints;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = qq.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("theoretical"));
            assert!(obj.contains_key("sample"));
            assert!(obj.contains_key("r_squared"));
            assert!(obj.contains_key("slope"));
            assert!(obj.contains_key("intercept"));

            // R² should be high for uniform data
            if let Some(Value::Number(r2)) = obj.get("r_squared") {
                let r2_val = r2.to_f64().unwrap_or(0.0);
                assert!(r2_val > 0.9, "R² should be high, got {}", r2_val);
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_qq_residuals() {
        let qq = QQResiduals;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = qq.call(&args, &ctx);

        if let Value::List(residuals) = result {
            assert_eq!(residuals.len(), 10);
            // Residuals should be small for reasonably normal data
            for r in residuals {
                if let Value::Number(_) = r {
                    // Just check it's a number
                } else {
                    panic!("Expected Number in residuals list");
                }
            }
        } else {
            panic!("Expected List, got {:?}", result);
        }
    }
}
