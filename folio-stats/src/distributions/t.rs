//! Student's t distribution functions

use folio_plugin::prelude::*;

// ============ T PDF ============

pub struct TPdf;

static T_PDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static T_PDF_EXAMPLES: [&str; 1] = ["t_pdf(0, 10) → 0.389..."];

static T_PDF_RELATED: [&str; 2] = ["t_cdf", "snorm_pdf"];

impl FunctionPlugin for TPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_pdf",
            description: "Student's t distribution PDF",
            usage: "t_pdf(x, df)",
            args: &T_PDF_ARGS,
            returns: "Number",
            examples: &T_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &T_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_pdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_pdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_pdf", "df", "Number", other.type_name())),
        };

        let df_f64 = df.to_f64().unwrap_or(1.0);
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("t_pdf() requires df > 0"));
        }

        let x_f64 = x.to_f64().unwrap_or(0.0);

        // t PDF using f64 for gamma function
        let result = t_pdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn t_pdf_f64(x: f64, df: f64) -> f64 {
    // PDF(x) = Γ((ν+1)/2) / (√(νπ) * Γ(ν/2)) * (1 + x²/ν)^(-(ν+1)/2)
    let nu = df;
    let coef = gamma_ln((nu + 1.0) / 2.0) - gamma_ln(nu / 2.0) - 0.5 * (nu * std::f64::consts::PI).ln();
    let term = -(nu + 1.0) / 2.0 * (1.0 + x * x / nu).ln();
    (coef + term).exp()
}

// ============ T CDF ============

pub struct TCdf;

static T_CDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static T_CDF_EXAMPLES: [&str; 1] = ["t_cdf(1.96, 30) → 0.97..."];

static T_CDF_RELATED: [&str; 2] = ["t_pdf", "t_inv"];

impl FunctionPlugin for TCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_cdf",
            description: "Student's t distribution CDF",
            usage: "t_cdf(x, df)",
            args: &T_CDF_ARGS,
            returns: "Number",
            examples: &T_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &T_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_cdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_cdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_cdf", "df", "Number", other.type_name())),
        };

        let df_f64 = df.to_f64().unwrap_or(1.0);
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("t_cdf() requires df > 0"));
        }

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let result = t_cdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

pub fn t_cdf_f64(x: f64, df: f64) -> f64 {
    // Use regularized incomplete beta function
    let t2 = x * x;
    let p = df / (df + t2);

    if x >= 0.0 {
        1.0 - 0.5 * regularized_incomplete_beta(df / 2.0, 0.5, p)
    } else {
        0.5 * regularized_incomplete_beta(df / 2.0, 0.5, p)
    }
}

// ============ T Inverse ============

pub struct TInv;

static T_INV_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static T_INV_EXAMPLES: [&str; 1] = ["t_inv(0.975, 30) → 2.042..."];

static T_INV_RELATED: [&str; 2] = ["t_cdf", "snorm_inv"];

impl FunctionPlugin for TInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_inv",
            description: "Student's t distribution inverse (quantile)",
            usage: "t_inv(p, df)",
            args: &T_INV_ARGS,
            returns: "Number",
            examples: &T_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &T_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_inv", 2, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_inv", "p", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_inv", "df", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("t_inv() requires 0 < p < 1"));
        }
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("t_inv() requires df > 0"));
        }

        let result = t_inv_f64(p_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn t_inv_f64(p: f64, df: f64) -> f64 {
    // Newton-Raphson iteration starting from normal approximation
    let mut x = norm_inv_approx(p);

    for _ in 0..50 {
        let cdf = t_cdf_f64(x, df);
        let pdf = t_pdf_f64(x, df);
        if pdf.abs() < 1e-15 {
            break;
        }
        let dx = (cdf - p) / pdf;
        x -= dx;
        if dx.abs() < 1e-12 {
            break;
        }
    }

    x
}

fn norm_inv_approx(p: f64) -> f64 {
    const A: [f64; 4] = [2.515517, 0.802853, 0.010328, 0.0];
    const B: [f64; 4] = [1.0, 1.432788, 0.189269, 0.001308];

    let sign = if p < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p < 0.5 { p } else { 1.0 - p };
    let t = (-2.0 * p_adj.ln()).sqrt();
    let num = A[0] + t * (A[1] + t * A[2]);
    let den = 1.0 + t * (B[1] + t * (B[2] + t * B[3]));
    sign * (t - num / den)
}

/// Log gamma function using Lanczos approximation
pub fn gamma_ln(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }

    const COEFFS: [f64; 8] = [
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    let g = 7.0;
    let z = x - 1.0;

    let mut sum = 0.99999999999980993;
    for (i, &c) in COEFFS.iter().enumerate() {
        sum += c / (z + i as f64 + 1.0);
    }

    let t = z + g + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + sum.ln()
}

/// Regularized incomplete beta function (simple approximation)
pub fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction for better accuracy
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (gamma_ln(a + b) - gamma_ln(a) - gamma_ln(b) + a * x.ln() + b * (1.0 - x).ln()).exp()
    };

    // Use continued fraction
    let sym = a / (a + b);
    if x < sym {
        bt * beta_cf(a, b, x) / a
    } else {
        1.0 - bt * beta_cf(b, a, 1.0 - x) / b
    }
}

fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let fpmin = 1e-30;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;

    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < fpmin {
        d = fpmin;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=200 {
        let m = m as f64;
        let m2 = 2.0 * m;

        // Even step
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = 1.0 + aa / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        h *= d * c;

        // Odd step
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = 1.0 + aa / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < 3e-14 {
            break;
        }
    }

    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_t_cdf_zero() {
        let t_cdf = TCdf;
        let args = vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(10)),
        ];
        let result = t_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        assert!((f - 0.5).abs() < 0.001);
    }
}
