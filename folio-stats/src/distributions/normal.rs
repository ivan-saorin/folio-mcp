//! Normal distribution functions

use folio_plugin::prelude::*;

// ============ Standard Normal PDF ============

pub struct SnormPdf;

static SNORM_PDF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "x",
    typ: "Number",
    description: "Value",
    optional: false,
    default: None,
}];

static SNORM_PDF_EXAMPLES: [&str; 1] = ["snorm_pdf(0) → 0.3989..."];

static SNORM_PDF_RELATED: [&str; 2] = ["snorm_cdf", "norm_pdf"];

impl FunctionPlugin for SnormPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "snorm_pdf",
            description: "Standard normal PDF (μ=0, σ=1)",
            usage: "snorm_pdf(x)",
            args: &SNORM_PDF_ARGS,
            returns: "Number",
            examples: &SNORM_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &SNORM_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("snorm_pdf", 1, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("snorm_pdf", "x", "Number", other.type_name())),
        };

        // PDF(x) = (1/√(2π)) * exp(-x²/2)
        let two = Number::from_i64(2);
        let pi = Number::pi(ctx.precision);
        let two_pi = two.mul(&pi);
        let sqrt_two_pi = match two_pi.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let x_squared = x.mul(x);
        let neg_half_x2 = Number::from_i64(0).sub(&x_squared.checked_div(&two).unwrap_or(Number::from_i64(0)));
        let exp_term = neg_half_x2.exp(ctx.precision);

        match exp_term.checked_div(&sqrt_two_pi) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Standard Normal CDF ============

pub struct SnormCdf;

static SNORM_CDF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "x",
    typ: "Number",
    description: "Value",
    optional: false,
    default: None,
}];

static SNORM_CDF_EXAMPLES: [&str; 1] = ["snorm_cdf(0) → 0.5"];

static SNORM_CDF_RELATED: [&str; 2] = ["snorm_pdf", "snorm_inv"];

impl FunctionPlugin for SnormCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "snorm_cdf",
            description: "Standard normal CDF P(X ≤ x)",
            usage: "snorm_cdf(x)",
            args: &SNORM_CDF_ARGS,
            returns: "Number",
            examples: &SNORM_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &SNORM_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("snorm_cdf", 1, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("snorm_cdf", "x", "Number", other.type_name())),
        };

        Value::Number(standard_normal_cdf(x, ctx.precision))
    }
}

/// Standard normal CDF using error function approximation
pub fn standard_normal_cdf(x: &Number, precision: u32) -> Number {
    // Φ(x) = 0.5 * (1 + erf(x/√2))
    let sqrt_2 = Number::from_i64(2).sqrt(precision).unwrap_or(Number::from_str("1.41421356").unwrap());
    let z = x.checked_div(&sqrt_2).unwrap_or(Number::from_i64(0));

    let erf_z = erf(&z, precision);
    let one = Number::from_i64(1);
    let half = Number::from_ratio(1, 2);

    half.mul(&one.add(&erf_z))
}

/// Error function approximation using Taylor series
fn erf(x: &Number, precision: u32) -> Number {
    // erf(x) ≈ (2/√π) * Σ((-1)^n * x^(2n+1)) / (n! * (2n+1))
    let x_f64 = x.to_f64().unwrap_or(0.0);

    // For large |x|, use asymptotic value
    if x_f64.abs() > 4.0 {
        if x_f64 > 0.0 {
            return Number::from_i64(1);
        } else {
            return Number::from_i64(-1);
        }
    }

    let pi = Number::pi(precision);
    let sqrt_pi = pi.sqrt(precision).unwrap_or(Number::from_str("1.77245385").unwrap());
    let two_over_sqrt_pi = Number::from_i64(2).checked_div(&sqrt_pi).unwrap_or(Number::from_i64(1));

    let mut sum = Number::from_i64(0);
    let mut term = x.clone();
    let x_squared = x.mul(x);

    let iterations = (precision / 2).max(20).min(100) as i64;

    for n in 0..iterations {
        // term = (-1)^n * x^(2n+1) / (n! * (2n+1))
        let divisor = Number::from_i64(2 * n + 1);
        let contribution = match term.checked_div(&divisor) {
            Ok(v) => v,
            Err(_) => break,
        };
        sum = sum.add(&contribution);

        // Next term: multiply by -x² / (n+1)
        let next_n = Number::from_i64(n + 1);
        term = Number::from_i64(0)
            .sub(&term.mul(&x_squared))
            .checked_div(&next_n)
            .unwrap_or(Number::from_i64(0));

        // Check for convergence
        if term.to_f64().map(|t| t.abs() < 1e-15).unwrap_or(true) {
            break;
        }
    }

    two_over_sqrt_pi.mul(&sum)
}

// ============ Standard Normal Inverse ============

pub struct SnormInv;

static SNORM_INV_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "p",
    typ: "Number",
    description: "Probability (0 < p < 1)",
    optional: false,
    default: None,
}];

static SNORM_INV_EXAMPLES: [&str; 1] = ["snorm_inv(0.975) → 1.96"];

static SNORM_INV_RELATED: [&str; 2] = ["snorm_cdf", "norm_inv"];

impl FunctionPlugin for SnormInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "snorm_inv",
            description: "Standard normal inverse (quantile function)",
            usage: "snorm_inv(p)",
            args: &SNORM_INV_ARGS,
            returns: "Number",
            examples: &SNORM_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &SNORM_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("snorm_inv", 1, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("snorm_inv", "p", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error(
                "snorm_inv() requires 0 < p < 1",
            ));
        }

        Value::Number(standard_normal_inv(p, ctx.precision))
    }
}

/// Standard normal inverse using rational approximation (Abramowitz and Stegun)
pub fn standard_normal_inv(p: &Number, _precision: u32) -> Number {
    let p_f64 = p.to_f64().unwrap_or(0.5);

    // Rational approximation constants
    const A: [f64; 4] = [2.515517, 0.802853, 0.010328, 0.0];
    const B: [f64; 4] = [1.0, 1.432788, 0.189269, 0.001308];

    let sign = if p_f64 < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p_f64 < 0.5 { p_f64 } else { 1.0 - p_f64 };

    let t = (-2.0 * p_adj.ln()).sqrt();

    let num = A[0] + t * (A[1] + t * A[2]);
    let den = 1.0 + t * (B[1] + t * (B[2] + t * B[3]));

    let result = sign * (t - num / den);

    // Convert to Number
    Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0))
}

// ============ Normal PDF ============

pub struct NormPdf;

static NORM_PDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ",
        typ: "Number",
        description: "Mean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "σ",
        typ: "Number",
        description: "Standard deviation (must be > 0)",
        optional: false,
        default: None,
    },
];

static NORM_PDF_EXAMPLES: [&str; 1] = ["norm_pdf(0, 0, 1) → 0.3989..."];

static NORM_PDF_RELATED: [&str; 2] = ["norm_cdf", "snorm_pdf"];

impl FunctionPlugin for NormPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm_pdf",
            description: "Normal distribution PDF",
            usage: "norm_pdf(x, μ, σ)",
            args: &NORM_PDF_ARGS,
            returns: "Number",
            examples: &NORM_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &NORM_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("norm_pdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_pdf", "x", "Number", other.type_name())),
        };

        let mu = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_pdf", "μ", "Number", other.type_name())),
        };

        let sigma = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_pdf", "σ", "Number", other.type_name())),
        };

        if sigma.is_zero() || sigma.is_negative() {
            return Value::Error(FolioError::domain_error("norm_pdf() requires σ > 0"));
        }

        // Standardize: z = (x - μ) / σ
        let z = match x.sub(mu).checked_div(sigma) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // PDF(x) = (1/(σ√(2π))) * exp(-(x-μ)²/(2σ²))
        let two = Number::from_i64(2);
        let pi = Number::pi(ctx.precision);
        let two_pi = two.mul(&pi);
        let sqrt_two_pi = match two_pi.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let z_squared = z.mul(&z);
        let neg_half_z2 = Number::from_i64(0).sub(&z_squared.checked_div(&two).unwrap_or(Number::from_i64(0)));
        let exp_term = neg_half_z2.exp(ctx.precision);

        let denominator = sigma.mul(&sqrt_two_pi);
        match exp_term.checked_div(&denominator) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Normal CDF ============

pub struct NormCdf;

static NORM_CDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ",
        typ: "Number",
        description: "Mean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "σ",
        typ: "Number",
        description: "Standard deviation (must be > 0)",
        optional: false,
        default: None,
    },
];

static NORM_CDF_EXAMPLES: [&str; 1] = ["norm_cdf(0, 0, 1) → 0.5"];

static NORM_CDF_RELATED: [&str; 2] = ["norm_pdf", "norm_inv"];

impl FunctionPlugin for NormCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm_cdf",
            description: "Normal distribution CDF P(X ≤ x)",
            usage: "norm_cdf(x, μ, σ)",
            args: &NORM_CDF_ARGS,
            returns: "Number",
            examples: &NORM_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &NORM_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("norm_cdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_cdf", "x", "Number", other.type_name())),
        };

        let mu = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_cdf", "μ", "Number", other.type_name())),
        };

        let sigma = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_cdf", "σ", "Number", other.type_name())),
        };

        if sigma.is_zero() || sigma.is_negative() {
            return Value::Error(FolioError::domain_error("norm_cdf() requires σ > 0"));
        }

        // Standardize: z = (x - μ) / σ
        let z = match x.sub(mu).checked_div(sigma) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        Value::Number(standard_normal_cdf(&z, ctx.precision))
    }
}

// ============ Normal Inverse ============

pub struct NormInv;

static NORM_INV_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ",
        typ: "Number",
        description: "Mean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "σ",
        typ: "Number",
        description: "Standard deviation (must be > 0)",
        optional: false,
        default: None,
    },
];

static NORM_INV_EXAMPLES: [&str; 1] = ["norm_inv(0.975, 0, 1) → 1.96"];

static NORM_INV_RELATED: [&str; 2] = ["norm_cdf", "snorm_inv"];

impl FunctionPlugin for NormInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm_inv",
            description: "Normal distribution inverse (quantile function)",
            usage: "norm_inv(p, μ, σ)",
            args: &NORM_INV_ARGS,
            returns: "Number",
            examples: &NORM_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &NORM_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("norm_inv", 3, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_inv", "p", "Number", other.type_name())),
        };

        let mu = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_inv", "μ", "Number", other.type_name())),
        };

        let sigma = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_inv", "σ", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("norm_inv() requires 0 < p < 1"));
        }

        if sigma.is_zero() || sigma.is_negative() {
            return Value::Error(FolioError::domain_error("norm_inv() requires σ > 0"));
        }

        // x = μ + σ * Φ⁻¹(p)
        let z = standard_normal_inv(p, ctx.precision);
        Value::Number(mu.add(&sigma.mul(&z)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_snorm_cdf_zero() {
        let snorm_cdf = SnormCdf;
        let args = vec![Value::Number(Number::from_i64(0))];
        let result = snorm_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        assert!((f - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_snorm_inv_half() {
        let snorm_inv = SnormInv;
        let args = vec![Value::Number(Number::from_str("0.5").unwrap())];
        let result = snorm_inv.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        assert!(f.abs() < 0.001);
    }
}
