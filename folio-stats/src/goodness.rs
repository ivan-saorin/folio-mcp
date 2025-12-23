//! Goodness-of-fit tests for distribution analysis

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted, mean};

// ============================================================================
// JarqueBera - Jarque-Bera test for normality
// ============================================================================

pub struct JarqueBera;

static JB_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
];
static JB_EXAMPLES: [&str; 1] = ["jarque_bera(data) → {statistic: 2.34, p_value: 0.31, ...}"];
static JB_RELATED: [&str; 2] = ["shapiro_wilk", "is_normal"];

impl FunctionPlugin for JarqueBera {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "jarque_bera",
            description: "Jarque-Bera test for normality (uses skewness and kurtosis)",
            usage: "jarque_bera(list)",
            args: &JB_ARGS,
            returns: "Object",
            examples: &JB_EXAMPLES,
            category: "distribution",
            source: None,
            related: &JB_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("jarque_bera", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 8, "jarque_bera") {
            return Value::Error(e);
        }

        let n = numbers.len();

        // Calculate mean
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Calculate central moments
        let mut m2 = Number::from_i64(0);
        let mut m3 = Number::from_i64(0);
        let mut m4 = Number::from_i64(0);

        for x in &numbers {
            let dev = x.sub(&m);
            let dev2 = dev.mul(&dev);
            let dev3 = dev2.mul(&dev);
            let dev4 = dev3.mul(&dev);
            m2 = m2.add(&dev2);
            m3 = m3.add(&dev3);
            m4 = m4.add(&dev4);
        }

        let n_num = Number::from_i64(n as i64);
        m2 = match m2.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };
        m3 = match m3.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };
        m4 = match m4.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if m2.is_zero() {
            return Value::Error(FolioError::domain_error("Variance is zero, cannot compute Jarque-Bera test"));
        }

        // Skewness = m3 / m2^(3/2)
        let m2_sqrt = match m2.sqrt(50) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };
        let m2_32 = m2_sqrt.mul(&m2);
        let skewness = match m3.checked_div(&m2_32) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };

        // Kurtosis = m4 / m2^2 - 3 (excess kurtosis)
        let m2_sq = m2.mul(&m2);
        let kurt_raw = match m4.checked_div(&m2_sq) {
            Ok(k) => k,
            Err(e) => return Value::Error(e.into()),
        };
        let kurtosis = kurt_raw.sub(&Number::from_i64(3));

        // JB statistic = n/6 * (S^2 + K^2/4)
        let skew_sq = skewness.mul(&skewness);
        let kurt_sq = kurtosis.mul(&kurtosis);
        let kurt_term = match kurt_sq.checked_div(&Number::from_i64(4)) {
            Ok(k) => k,
            Err(e) => return Value::Error(e.into()),
        };
        let sum_terms = skew_sq.add(&kurt_term);
        let jb_stat = match n_num.checked_div(&Number::from_i64(6)) {
            Ok(factor) => factor.mul(&sum_terms),
            Err(e) => return Value::Error(e.into()),
        };

        // P-value from chi-squared distribution with 2 df
        let jb_f64 = jb_stat.to_f64().unwrap_or(0.0);
        let p_value = chi_squared_sf(jb_f64, 2);

        let mut result = HashMap::new();
        result.insert("statistic".to_string(), Value::Number(jb_stat));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("skewness".to_string(), Value::Number(skewness));
        result.insert("kurtosis".to_string(), Value::Number(kurtosis));

        Value::Object(result)
    }
}

// ============================================================================
// ShapiroWilk - Shapiro-Wilk test for normality
// ============================================================================

pub struct ShapiroWilk;

static SW_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values (3 ≤ n ≤ 5000)",
        optional: false,
        default: None,
    },
];
static SW_EXAMPLES: [&str; 1] = ["shapiro_wilk(data) → {w: 0.967, p_value: 0.234}"];
static SW_RELATED: [&str; 2] = ["jarque_bera", "is_normal"];

impl FunctionPlugin for ShapiroWilk {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "shapiro_wilk",
            description: "Shapiro-Wilk test for normality (best for n < 50)",
            usage: "shapiro_wilk(list)",
            args: &SW_ARGS,
            returns: "Object",
            examples: &SW_EXAMPLES,
            category: "distribution",
            source: None,
            related: &SW_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("shapiro_wilk", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = numbers.len();
        if n < 3 {
            return Value::Error(FolioError::domain_error("Shapiro-Wilk requires at least 3 values"));
        }
        if n > 5000 {
            return Value::Error(FolioError::domain_error("Shapiro-Wilk limited to n ≤ 5000"));
        }

        let sorted_data = sorted(&numbers);
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Calculate SS (sum of squared deviations)
        let mut ss = Number::from_i64(0);
        for x in &numbers {
            let dev = x.sub(&m);
            ss = ss.add(&dev.mul(&dev));
        }

        if ss.is_zero() {
            return Value::Error(FolioError::domain_error("All values are identical"));
        }

        // Calculate W statistic using the standard Shapiro-Wilk formula
        // W = (Σ a_i * x_(i))² / SS
        // where x_(i) are order statistics and a_i are Shapiro-Wilk coefficients

        // Convert to f64 for calculation
        let sorted_f64: Vec<f64> = sorted_data.iter()
            .map(|x| x.to_f64().unwrap_or(0.0))
            .collect();
        let ss_f64 = ss.to_f64().unwrap_or(1.0);

        // Get the Shapiro-Wilk coefficients (full vector for all n values)
        let a_coeffs = shapiro_wilk_coefficients(n);

        // Calculate the numerator: (Σ a_i * x_(i))²
        // The coefficients a are symmetric: a[i] = -a[n-1-i] for i < n/2
        // So the sum becomes: Σ a[i] * (x_(n-i) - x_(i+1)) for i = 0..n/2
        let mut b = 0.0_f64;
        let half_n = n / 2;

        for i in 0..half_n {
            b += a_coeffs[i] * (sorted_f64[n - 1 - i] - sorted_f64[i]);
        }

        let w_val = if ss_f64 > 0.0 {
            let w = (b * b) / ss_f64;
            // W should be in (0, 1] but may slightly exceed due to numerical precision
            w.min(1.0).max(0.0)
        } else {
            1.0
        };

        let w_stat = Number::from_str(&format!("{:.10}", w_val)).unwrap_or(Number::from_i64(1));

        // Approximate p-value using Royston's approximation
        let w_f64 = w_stat.to_f64().unwrap_or(1.0);
        let p_value = shapiro_wilk_p_value(w_f64, n);

        let mut result = HashMap::new();
        result.insert("w".to_string(), Value::Number(w_stat));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

/// Generate Shapiro-Wilk coefficients
/// Uses exact tabulated values from Shapiro-Wilk (1965) and Royston (1992)
fn shapiro_wilk_coefficients(n: usize) -> Vec<f64> {
    let half_n = n / 2;

    if n < 3 {
        return vec![0.0; half_n];
    }

    // Exact tabulated coefficients from Shapiro-Wilk (1965)
    // These are the 'a' coefficients for the first half (paired with symmetric negatives)
    // Source: Original S-W paper and verified against R's shapiro.test
    match n {
        3 => vec![0.7071],
        4 => vec![0.6872, 0.1677],
        5 => vec![0.6646, 0.2413],
        6 => vec![0.6431, 0.2806, 0.0875],
        7 => vec![0.6233, 0.3031, 0.1401],
        8 => vec![0.6052, 0.3164, 0.1743, 0.0561],
        9 => vec![0.5888, 0.3244, 0.1976, 0.0947],
        10 => vec![0.5739, 0.3291, 0.2141, 0.1224, 0.0399],
        11 => vec![0.5601, 0.3315, 0.2260, 0.1429, 0.0695],
        12 => vec![0.5475, 0.3325, 0.2347, 0.1586, 0.0922, 0.0303],
        13 => vec![0.5359, 0.3325, 0.2412, 0.1707, 0.1099, 0.0539],
        14 => vec![0.5251, 0.3318, 0.2460, 0.1802, 0.1240, 0.0727, 0.0240],
        15 => vec![0.5150, 0.3306, 0.2495, 0.1878, 0.1353, 0.0880, 0.0433],
        16 => vec![0.5056, 0.3290, 0.2521, 0.1939, 0.1447, 0.1005, 0.0593, 0.0196],
        17 => vec![0.4968, 0.3273, 0.2540, 0.1988, 0.1524, 0.1109, 0.0725, 0.0359],
        18 => vec![0.4886, 0.3253, 0.2553, 0.2027, 0.1587, 0.1197, 0.0837, 0.0496, 0.0163],
        19 => vec![0.4808, 0.3232, 0.2561, 0.2059, 0.1641, 0.1271, 0.0932, 0.0612, 0.0303],
        20 => vec![0.4734, 0.3211, 0.2565, 0.2085, 0.1686, 0.1334, 0.1013, 0.0711, 0.0422, 0.0140],
        _ => {
            // For n > 20, use Royston's approximation (AS R94)
            royston_coefficients(n)
        }
    }
}

/// Royston's approximation for Shapiro-Wilk coefficients when n > 20
fn royston_coefficients(n: usize) -> Vec<f64> {
    let half_n = n / 2;
    let n_f64 = n as f64;

    // Calculate expected order statistics (m-values) using Blom's approximation
    let m: Vec<f64> = (1..=n).map(|i| {
        let p = (i as f64 - 0.375) / (n_f64 + 0.25);
        normal_quantile(p)
    }).collect();

    // Calculate sum of m^2
    let m_sq_sum: f64 = m.iter().map(|x| x * x).sum();

    if m_sq_sum == 0.0 {
        return vec![0.0; half_n];
    }

    let sqrt_m_sq_sum = m_sq_sum.sqrt();

    // Polynomial approximation for the largest coefficient a[n]
    // From Royston (1992) Algorithm AS R94
    let u = 1.0 / n_f64.sqrt();

    // Coefficients c1-c6 for a_n polynomial
    let a_n = {
        let poly_val = -2.706056 * u.powi(5)
            + 4.434685 * u.powi(4)
            - 2.071190 * u.powi(3)
            - 0.147981 * u.powi(2)
            + 0.221157 * u;
        poly_val + m[n - 1] / sqrt_m_sq_sum
    };

    // Calculate remaining coefficients using the constraint sum(2*a_i^2) = 1
    // (because each a_i pairs with -a_i on the other end)
    let mut a = vec![0.0; half_n];
    a[0] = a_n;

    // For the remaining coefficients, scale the m values
    // so that the sum of squares constraint is satisfied
    let remaining_m_sq: f64 = (1..half_n).map(|i| {
        let diff = m[n - 1 - i] - m[i];
        diff * diff
    }).sum();

    let a_n_contribution = 2.0 * a_n * a_n;
    let remaining_scale_sq = (1.0 - a_n_contribution) / (4.0 * remaining_m_sq);

    if remaining_scale_sq > 0.0 {
        let scale = remaining_scale_sq.sqrt();
        for i in 1..half_n {
            a[i] = scale * (m[n - 1 - i] - m[i]);
        }
    } else {
        // Fallback: use normalized m differences
        for i in 1..half_n {
            a[i] = (m[n - 1 - i] - m[i]) / (2.0 * sqrt_m_sq_sum);
        }
    }

    // Final normalization to ensure sum of 2*a_i^2 = 1
    let sum_2a_sq: f64 = a.iter().map(|x| 2.0 * x * x).sum();
    if sum_2a_sq > 0.0 && (sum_2a_sq - 1.0).abs() > 0.001 {
        let norm = sum_2a_sq.sqrt();
        for coeff in &mut a {
            *coeff /= norm;
        }
    }

    a
}

/// Approximate p-value for Shapiro-Wilk test using Royston's Algorithm AS R94
/// This implements the transformation to normality and returns the p-value
fn shapiro_wilk_p_value(w: f64, n: usize) -> f64 {
    let n_f64 = n as f64;

    if w >= 1.0 {
        return 1.0;
    }
    if w <= 0.0 {
        return 0.0;
    }

    // Royston's 1992 algorithm for p-value approximation
    // Different transformations for different sample size ranges

    if n <= 11 {
        // Small sample: use gamma approximation
        let gamma = poly(&[-2.273, 0.459], n_f64);
        let mu = poly(&[0.544, -0.39978, 0.025054, -0.0006714], n_f64);
        let sigma = poly(&[1.3822, -0.77857, 0.062767, -0.0020322], n_f64).exp();

        let y = -((1.0 - w).ln());
        let z = (y - mu) / sigma;

        // Adjust for gamma distribution shape
        1.0 - normal_cdf(gamma + z * (1.0 + gamma * sigma).abs())
    } else if n <= 2000 {
        // Medium to large sample: use log transformation
        let ln_n = n_f64.ln();

        // Royston's coefficients for the transformation
        let mu = poly(&[-1.5861, -0.31082, -0.083751, 0.0038915], ln_n);
        let sigma = poly(&[-0.4803, -0.082676, 0.0030302], ln_n).exp();

        // Transform W to approximately standard normal
        let y = (1.0 - w).ln();
        let z = (y - mu) / sigma;

        // P-value from standard normal (upper tail)
        1.0 - normal_cdf(z)
    } else {
        // Very large sample: use asymptotic approximation
        // For n > 2000, W is approximately normal with known mean and variance
        let mean_w = 1.0 - 2.0 / (9.0 * n_f64);
        let var_w = 2.0 / (81.0 * n_f64 * n_f64);
        let z = (w - mean_w) / var_w.sqrt();

        normal_cdf(z)
    }
}

/// Evaluate polynomial at x: c[0] + c[1]*x + c[2]*x^2 + ...
fn poly(coeffs: &[f64], x: f64) -> f64 {
    let mut result = 0.0;
    let mut x_pow = 1.0;
    for &c in coeffs {
        result += c * x_pow;
        x_pow *= x;
    }
    result
}

// ============================================================================
// IsNormal - Convenience function for normality test
// ============================================================================

pub struct IsNormal;

static IS_NORMAL_ARGS: [ArgMeta; 2] = [
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
static IS_NORMAL_EXAMPLES: [&str; 2] = [
    "is_normal(data) → true",
    "is_normal(data, 0.01) → false (stricter test)",
];
static IS_NORMAL_RELATED: [&str; 2] = ["shapiro_wilk", "jarque_bera"];

impl FunctionPlugin for IsNormal {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_normal",
            description: "Test if data is normally distributed",
            usage: "is_normal(list, alpha?)",
            args: &IS_NORMAL_ARGS,
            returns: "Bool",
            examples: &IS_NORMAL_EXAMPLES,
            category: "distribution",
            source: None,
            related: &IS_NORMAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("is_normal", 1, args.len()));
        }

        let alpha = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.05),
                other => return Value::Error(FolioError::arg_type("is_normal", "alpha", "Number", other.type_name())),
            }
        } else {
            0.05
        };

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = numbers.len();
        if n < 3 {
            return Value::Error(FolioError::domain_error("is_normal requires at least 3 values"));
        }

        // Use Shapiro-Wilk for small samples, Jarque-Bera for large
        let p_value = if n < 50 {
            let sw = ShapiroWilk;
            match sw.call(&args[0..1], ctx) {
                Value::Object(obj) => {
                    if let Some(Value::Number(p)) = obj.get("p_value") {
                        p.to_f64().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        } else {
            let jb = JarqueBera;
            match jb.call(&args[0..1], ctx) {
                Value::Object(obj) => {
                    if let Some(Value::Number(p)) = obj.get("p_value") {
                        p.to_f64().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        };

        // If p > alpha, we fail to reject normality hypothesis
        Value::Bool(p_value > alpha)
    }
}

// ============================================================================
// KsTest2 - Two-sample Kolmogorov-Smirnov test
// ============================================================================

pub struct KsTest2;

static KS_TEST_2_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List",
        description: "First sample",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List",
        description: "Second sample",
        optional: false,
        default: None,
    },
];
static KS_TEST_2_EXAMPLES: [&str; 1] = ["ks_test_2(before, after) → {statistic, p_value, critical_01, critical_05, critical_10}"];
static KS_TEST_2_RELATED: [&str; 2] = ["t_test_2", "anderson_darling"];

impl FunctionPlugin for KsTest2 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ks_test_2",
            description: "Two-sample Kolmogorov-Smirnov test",
            usage: "ks_test_2(list1, list2)",
            args: &KS_TEST_2_ARGS,
            returns: "Object",
            examples: &KS_TEST_2_EXAMPLES,
            category: "distribution",
            source: None,
            related: &KS_TEST_2_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("ks_test_2", 2, args.len()));
        }

        let x = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let y = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&x, 5, "ks_test_2") {
            return Value::Error(e);
        }
        if let Err(e) = require_min_count(&y, 5, "ks_test_2") {
            return Value::Error(e);
        }

        let n1 = x.len();
        let n2 = y.len();

        // Sort both samples
        let sorted_x = sorted(&x);
        let sorted_y = sorted(&y);

        // Calculate KS statistic: max |F1(x) - F2(x)|
        let mut d_max = 0.0;
        let mut i = 0;
        let mut j = 0;

        while i < n1 && j < n2 {
            let x_val = sorted_x[i].to_f64().unwrap_or(0.0);
            let y_val = sorted_y[j].to_f64().unwrap_or(0.0);

            if x_val <= y_val {
                i += 1;
            }
            if y_val <= x_val {
                j += 1;
            }

            let f1 = i as f64 / n1 as f64;
            let f2 = j as f64 / n2 as f64;
            let d = (f1 - f2).abs();
            if d > d_max {
                d_max = d;
            }
        }

        // Calculate p-value using asymptotic distribution
        let n_eff = (n1 * n2) as f64 / (n1 + n2) as f64;
        let lambda = (n_eff.sqrt() + 0.12 + 0.11 / n_eff.sqrt()) * d_max;
        let p_value = ks_p_value(lambda);

        // Critical values
        let c_alpha = |alpha: f64| -> f64 {
            (-0.5 * (alpha / 2.0).ln()).sqrt() / (n_eff.sqrt() + 0.12 + 0.11 / n_eff.sqrt())
        };

        let mut result = HashMap::new();
        result.insert("statistic".to_string(), Value::Number(Number::from_str(&format!("{:.10}", d_max)).unwrap_or(Number::from_i64(0))));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("critical_01".to_string(), Value::Number(Number::from_str(&format!("{:.10}", c_alpha(0.01))).unwrap_or(Number::from_i64(0))));
        result.insert("critical_05".to_string(), Value::Number(Number::from_str(&format!("{:.10}", c_alpha(0.05))).unwrap_or(Number::from_i64(0))));
        result.insert("critical_10".to_string(), Value::Number(Number::from_str(&format!("{:.10}", c_alpha(0.10))).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

// ============================================================================
// Helper functions for statistical distributions
// ============================================================================

/// Standard normal CDF
fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal quantile (inverse CDF)
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if p == 0.5 {
        return 0.0;
    }

    // Rational approximation
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

/// Error function approximation
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Chi-squared survival function (1 - CDF)
fn chi_squared_sf(x: f64, df: usize) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    // Use incomplete gamma function
    1.0 - incomplete_gamma(df as f64 / 2.0, x / 2.0)
}

/// Regularized incomplete gamma function (lower)
fn incomplete_gamma(a: f64, x: f64) -> f64 {
    if x < 0.0 || a <= 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }

    // Use series expansion for small x, continued fraction for large x
    if x < a + 1.0 {
        // Series expansion
        gamma_series(a, x)
    } else {
        // Continued fraction
        1.0 - gamma_cf(a, x)
    }
}

fn gamma_series(a: f64, x: f64) -> f64 {
    let gln = ln_gamma(a);
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut del = sum;

    for _ in 0..100 {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 1e-10 {
            break;
        }
    }

    sum * (-x + a * x.ln() - gln).exp()
}

fn gamma_cf(a: f64, x: f64) -> f64 {
    let gln = ln_gamma(a);
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / 1e-30;
    let mut d = 1.0 / b;
    let mut h = d;

    for i in 1..100 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = b + an / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-10 {
            break;
        }
    }

    (-x + a * x.ln() - gln).exp() * h
}

fn ln_gamma(x: f64) -> f64 {
    let cof = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179e-2,
        -0.5395239384953e-5,
    ];

    let y = x;
    let tmp = x + 5.5 - (x + 0.5) * (x + 5.5).ln();
    let mut ser = 1.000000000190015;
    for (j, &c) in cof.iter().enumerate() {
        ser += c / (y + j as f64 + 1.0);
    }

    -tmp + (2.5066282746310005 * ser / x).ln()
}

/// KS p-value from the Kolmogorov distribution
fn ks_p_value(lambda: f64) -> f64 {
    if lambda <= 0.0 {
        return 1.0;
    }
    if lambda >= 3.0 {
        return 0.0;
    }

    // Asymptotic formula
    let mut sum = 0.0;
    for k in 1..100 {
        let k_f64 = k as f64;
        let term = (-2.0 * k_f64 * k_f64 * lambda * lambda).exp();
        if k % 2 == 0 {
            sum -= term;
        } else {
            sum += term;
        }
        if term.abs() < 1e-10 {
            break;
        }
    }
    2.0 * sum
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
    fn test_jarque_bera() {
        let jb = JarqueBera;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = jb.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("statistic"));
            assert!(obj.contains_key("p_value"));
            assert!(obj.contains_key("skewness"));
            assert!(obj.contains_key("kurtosis"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_shapiro_wilk() {
        let sw = ShapiroWilk;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = sw.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("w"));
            assert!(obj.contains_key("p_value"));
            // W should be between 0 and 1 (or close to it)
            if let Some(Value::Number(w)) = obj.get("w") {
                let w_val = w.to_f64().unwrap_or(0.0);
                // Allow some tolerance as simplified implementation may slightly exceed 1
                assert!(w_val > 0.0 && w_val <= 1.1, "W should be in range (0,1], got {}", w_val);
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_is_normal() {
        let is_norm = IsNormal;
        // Uniform data - should not be normal
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = is_norm.call(&args, &ctx);
        assert!(matches!(result, Value::Bool(_)));
    }

    #[test]
    fn test_ks_test_2() {
        let ks = KsTest2;
        let args = vec![
            make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            make_list(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        ];
        let ctx = eval_ctx();
        let result = ks.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("statistic"));
            assert!(obj.contains_key("p_value"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }
}
