//! Outlier detection functions

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted, mean, variance_impl};

// ============================================================================
// OutliersIqr - IQR-based outlier detection
// ============================================================================

pub struct OutliersIqr;

static OUTLIERS_IQR_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "IQR multiplier (default 1.5, use 3 for extreme)",
        optional: true,
        default: Some("1.5"),
    },
];
static OUTLIERS_IQR_EXAMPLES: [&str; 2] = [
    "outliers_iqr([1,2,3,100]) → {indices: [...], values: [...], lower_fence, upper_fence, q1, q3, iqr, count}",
    "outliers_iqr(data, 3) → extreme outliers only (k=3)",
];
static OUTLIERS_IQR_RELATED: [&str; 2] = ["outliers_zscore", "outliers_mad"];

impl FunctionPlugin for OutliersIqr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outliers_iqr",
            description: "IQR-based outlier detection (Tukey's method)",
            usage: "outliers_iqr(list, k?)",
            args: &OUTLIERS_IQR_ARGS,
            returns: "Object",
            examples: &OUTLIERS_IQR_EXAMPLES,
            category: "distribution",
            source: None,
            related: &OUTLIERS_IQR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("outliers_iqr", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 4, "outliers_iqr") {
            return Value::Error(e);
        }

        // Get k multiplier (default 1.5)
        let k = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.clone(),
                other => return Value::Error(FolioError::arg_type("outliers_iqr", "k", "Number", other.type_name())),
            }
        } else {
            Number::from_str("1.5").unwrap()
        };

        // Calculate Q1, Q3, IQR
        let q1 = match crate::helpers::percentile_impl(&numbers, &Number::from_i64(25)) {
            Ok(q) => q,
            Err(e) => return Value::Error(e),
        };
        let q3 = match crate::helpers::percentile_impl(&numbers, &Number::from_i64(75)) {
            Ok(q) => q,
            Err(e) => return Value::Error(e),
        };
        let iqr = q3.sub(&q1);

        // Calculate fences
        let k_iqr = k.mul(&iqr);
        let lower_fence = q1.sub(&k_iqr);
        let upper_fence = q3.add(&k_iqr);

        // Find outliers
        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (i, x) in numbers.iter().enumerate() {
            let below_lower = x.sub(&lower_fence).is_negative();
            let above_upper = !x.sub(&upper_fence).is_negative() && !x.sub(&upper_fence).is_zero();
            if below_lower || above_upper {
                indices.push(Value::Number(Number::from_i64(i as i64)));
                values.push(Value::Number(x.clone()));
            }
        }

        let count = indices.len() as i64;

        let mut result = HashMap::new();
        result.insert("indices".to_string(), Value::List(indices));
        result.insert("values".to_string(), Value::List(values));
        result.insert("count".to_string(), Value::Number(Number::from_i64(count)));
        result.insert("lower_fence".to_string(), Value::Number(lower_fence));
        result.insert("upper_fence".to_string(), Value::Number(upper_fence));
        result.insert("q1".to_string(), Value::Number(q1));
        result.insert("q3".to_string(), Value::Number(q3));
        result.insert("iqr".to_string(), Value::Number(iqr));

        Value::Object(result)
    }
}

// ============================================================================
// OutliersZscore - Z-score based outlier detection
// ============================================================================

pub struct OutliersZscore;

static OUTLIERS_ZSCORE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "threshold",
        typ: "Number",
        description: "Z-score threshold (default 3)",
        optional: true,
        default: Some("3"),
    },
];
static OUTLIERS_ZSCORE_EXAMPLES: [&str; 1] = ["outliers_zscore([1,2,3,100], 3) → {indices: [3], z_scores: [2.89], ...}"];
static OUTLIERS_ZSCORE_RELATED: [&str; 2] = ["outliers_iqr", "zscore"];

impl FunctionPlugin for OutliersZscore {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outliers_zscore",
            description: "Z-score based outlier detection",
            usage: "outliers_zscore(list, threshold?)",
            args: &OUTLIERS_ZSCORE_ARGS,
            returns: "Object",
            examples: &OUTLIERS_ZSCORE_EXAMPLES,
            category: "distribution",
            source: None,
            related: &OUTLIERS_ZSCORE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("outliers_zscore", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "outliers_zscore") {
            return Value::Error(e);
        }

        // Get threshold (default 3)
        let threshold = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.clone(),
                other => return Value::Error(FolioError::arg_type("outliers_zscore", "threshold", "Number", other.type_name())),
            }
        } else {
            Number::from_i64(3)
        };

        // Calculate mean and stddev
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
            // All values are the same, no outliers
            let mut result = HashMap::new();
            result.insert("indices".to_string(), Value::List(vec![]));
            result.insert("values".to_string(), Value::List(vec![]));
            result.insert("z_scores".to_string(), Value::List(vec![]));
            result.insert("count".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("mean".to_string(), Value::Number(m));
            result.insert("stddev".to_string(), Value::Number(stddev));
            result.insert("threshold".to_string(), Value::Number(threshold));
            return Value::Object(result);
        }

        // Find outliers
        let mut indices = Vec::new();
        let mut values = Vec::new();
        let mut z_scores = Vec::new();

        for (i, x) in numbers.iter().enumerate() {
            let z = match x.sub(&m).checked_div(&stddev) {
                Ok(z) => z,
                Err(_) => continue,
            };
            let abs_z = z.abs();
            if !abs_z.sub(&threshold).is_negative() {
                indices.push(Value::Number(Number::from_i64(i as i64)));
                values.push(Value::Number(x.clone()));
                z_scores.push(Value::Number(z));
            }
        }

        let count = indices.len() as i64;

        let mut result = HashMap::new();
        result.insert("indices".to_string(), Value::List(indices));
        result.insert("values".to_string(), Value::List(values));
        result.insert("z_scores".to_string(), Value::List(z_scores));
        result.insert("count".to_string(), Value::Number(Number::from_i64(count)));
        result.insert("mean".to_string(), Value::Number(m));
        result.insert("stddev".to_string(), Value::Number(stddev));
        result.insert("threshold".to_string(), Value::Number(threshold));

        Value::Object(result)
    }
}

// ============================================================================
// OutliersMad - MAD-based outlier detection (robust)
// ============================================================================

pub struct OutliersMad;

static OUTLIERS_MAD_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "threshold",
        typ: "Number",
        description: "Modified Z-score threshold (default 3.5)",
        optional: true,
        default: Some("3.5"),
    },
];
static OUTLIERS_MAD_EXAMPLES: [&str; 1] = ["outliers_mad([1,2,3,100], 3.5) → {indices: [...], modified_z: [...], ...}"];
static OUTLIERS_MAD_RELATED: [&str; 2] = ["outliers_iqr", "mad"];

impl FunctionPlugin for OutliersMad {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outliers_mad",
            description: "MAD-based outlier detection (robust to outliers)",
            usage: "outliers_mad(list, threshold?)",
            args: &OUTLIERS_MAD_ARGS,
            returns: "Object",
            examples: &OUTLIERS_MAD_EXAMPLES,
            category: "distribution",
            source: None,
            related: &OUTLIERS_MAD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("outliers_mad", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "outliers_mad") {
            return Value::Error(e);
        }

        // Get threshold (default 3.5)
        let threshold = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.clone(),
                other => return Value::Error(FolioError::arg_type("outliers_mad", "threshold", "Number", other.type_name())),
            }
        } else {
            Number::from_str("3.5").unwrap()
        };

        // Calculate median
        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();
        let median = if n % 2 == 0 {
            let mid = n / 2;
            sorted_nums[mid - 1].add(&sorted_nums[mid]).checked_div(&Number::from_i64(2)).unwrap_or(sorted_nums[mid].clone())
        } else {
            sorted_nums[n / 2].clone()
        };

        // Calculate MAD (Median Absolute Deviation)
        let abs_devs: Vec<Number> = numbers.iter().map(|x| x.sub(&median).abs()).collect();
        let sorted_devs = sorted(&abs_devs);
        let mad = if n % 2 == 0 {
            let mid = n / 2;
            sorted_devs[mid - 1].add(&sorted_devs[mid]).checked_div(&Number::from_i64(2)).unwrap_or(sorted_devs[mid].clone())
        } else {
            sorted_devs[n / 2].clone()
        };

        // Scale factor for consistency with normal distribution: 1.4826
        let scale = Number::from_str("1.4826").unwrap();
        let scaled_mad = mad.mul(&scale);

        if scaled_mad.is_zero() {
            // All values have same deviation from median
            let mut result = HashMap::new();
            result.insert("indices".to_string(), Value::List(vec![]));
            result.insert("values".to_string(), Value::List(vec![]));
            result.insert("modified_z".to_string(), Value::List(vec![]));
            result.insert("count".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("median".to_string(), Value::Number(median));
            result.insert("mad".to_string(), Value::Number(mad));
            result.insert("threshold".to_string(), Value::Number(threshold));
            return Value::Object(result);
        }

        // Find outliers using modified Z-score
        let mut indices = Vec::new();
        let mut values = Vec::new();
        let mut modified_z = Vec::new();

        for (i, x) in numbers.iter().enumerate() {
            let m_z = match x.sub(&median).checked_div(&scaled_mad) {
                Ok(z) => z,
                Err(_) => continue,
            };
            let abs_mz = m_z.abs();
            if !abs_mz.sub(&threshold).is_negative() {
                indices.push(Value::Number(Number::from_i64(i as i64)));
                values.push(Value::Number(x.clone()));
                modified_z.push(Value::Number(m_z));
            }
        }

        let count = indices.len() as i64;

        let mut result = HashMap::new();
        result.insert("indices".to_string(), Value::List(indices));
        result.insert("values".to_string(), Value::List(values));
        result.insert("modified_z".to_string(), Value::List(modified_z));
        result.insert("count".to_string(), Value::Number(Number::from_i64(count)));
        result.insert("median".to_string(), Value::Number(median));
        result.insert("mad".to_string(), Value::Number(mad));
        result.insert("threshold".to_string(), Value::Number(threshold));

        Value::Object(result)
    }
}

// ============================================================================
// GrubbsTest - Grubbs' test for single outlier
// ============================================================================

pub struct GrubbsTest;

static GRUBBS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "alpha",
        typ: "Number",
        description: "Significance level (default 0.05)",
        optional: true,
        default: Some("0.05"),
    },
];
static GRUBBS_EXAMPLES: [&str; 1] = ["grubbs_test([1,2,3,100]) → {has_outlier: true, outlier, index, g_statistic, critical_value, p_value}"];
static GRUBBS_RELATED: [&str; 2] = ["outliers_iqr", "outliers_zscore"];

impl FunctionPlugin for GrubbsTest {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "grubbs_test",
            description: "Grubbs' test for a single outlier",
            usage: "grubbs_test(list, alpha?)",
            args: &GRUBBS_ARGS,
            returns: "Object",
            examples: &GRUBBS_EXAMPLES,
            category: "distribution",
            source: None,
            related: &GRUBBS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("grubbs_test", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "grubbs_test") {
            return Value::Error(e);
        }

        // Get alpha (default 0.05)
        let alpha = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.05),
                other => return Value::Error(FolioError::arg_type("grubbs_test", "alpha", "Number", other.type_name())),
            }
        } else {
            0.05
        };

        let n = numbers.len();

        // Calculate mean and stddev
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
            let mut result = HashMap::new();
            result.insert("has_outlier".to_string(), Value::Bool(false));
            result.insert("outlier".to_string(), Value::Null);
            result.insert("index".to_string(), Value::Null);
            result.insert("g_statistic".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("critical_value".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("p_value".to_string(), Value::Number(Number::from_i64(1)));
            return Value::Object(result);
        }

        // Find the value with maximum deviation from mean
        let mut max_dev = Number::from_i64(0);
        let mut outlier_idx = 0;
        let mut outlier_val = numbers[0].clone();

        for (i, x) in numbers.iter().enumerate() {
            let dev = x.sub(&m).abs();
            if !dev.sub(&max_dev).is_negative() && !dev.sub(&max_dev).is_zero() {
                max_dev = dev.clone();
                outlier_idx = i;
                outlier_val = x.clone();
            }
        }

        // Calculate Grubbs statistic G = max|x - mean| / s
        let g_stat = match max_dev.checked_div(&stddev) {
            Ok(g) => g,
            Err(e) => return Value::Error(e.into()),
        };

        // Calculate critical value using t-distribution approximation
        // G_crit = ((n-1) / sqrt(n)) * sqrt(t^2 / (n - 2 + t^2))
        // where t is the two-sided t-value at alpha/(2n) with n-2 df
        let n_f64 = n as f64;

        // Approximate t-value using inverse normal (simplified for common cases)
        let t_alpha = t_critical(alpha / (2.0 * n_f64), n - 2);
        let t_sq = t_alpha * t_alpha;
        let g_crit = ((n_f64 - 1.0) / n_f64.sqrt()) * (t_sq / (n_f64 - 2.0 + t_sq)).sqrt();

        let has_outlier = g_stat.to_f64().unwrap_or(0.0) > g_crit;

        // Approximate p-value (simplified)
        let p_value = if has_outlier { alpha } else { 1.0 - alpha };

        let mut result = HashMap::new();
        result.insert("has_outlier".to_string(), Value::Bool(has_outlier));
        if has_outlier {
            result.insert("outlier".to_string(), Value::Number(outlier_val));
            result.insert("index".to_string(), Value::Number(Number::from_i64(outlier_idx as i64)));
        } else {
            result.insert("outlier".to_string(), Value::Null);
            result.insert("index".to_string(), Value::Null);
        }
        result.insert("g_statistic".to_string(), Value::Number(g_stat));
        result.insert("critical_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", g_crit)).unwrap_or(Number::from_i64(0))));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

/// Approximate t-critical value using inverse normal approximation
fn t_critical(alpha: f64, df: usize) -> f64 {
    // Use normal approximation for large df, else lookup tables
    // This is a simplified approximation
    let z = normal_inv(1.0 - alpha);
    if df >= 30 {
        z
    } else {
        // Cornish-Fisher expansion for small df
        let g1 = (z * z * z + z) / 4.0;
        let g2 = (5.0 * z.powi(5) + 16.0 * z.powi(3) + 3.0 * z) / 96.0;
        z + g1 / (df as f64) + g2 / ((df as f64).powi(2))
    }
}

/// Approximate inverse normal CDF
fn normal_inv(p: f64) -> f64 {
    // Abramowitz and Stegun approximation
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if p == 0.5 {
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
    fn test_outliers_iqr() {
        let outliers = OutliersIqr;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 100])];
        let ctx = eval_ctx();
        let result = outliers.call(&args, &ctx);

        if let Value::Object(obj) = result {
            if let Some(Value::Number(count)) = obj.get("count") {
                assert!(count.to_i64().unwrap() > 0, "Should detect outlier");
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_outliers_zscore() {
        let outliers = OutliersZscore;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 100])];
        let ctx = eval_ctx();
        let result = outliers.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("indices"));
            assert!(obj.contains_key("z_scores"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }
}
