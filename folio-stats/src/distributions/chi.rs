//! Chi-squared distribution functions

use folio_plugin::prelude::*;
use super::t::gamma_ln;

// ============ Chi PDF ============

pub struct ChiPdf;

static CHI_PDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
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

static CHI_PDF_EXAMPLES: [&str; 1] = ["chi_pdf(5, 3) → 0.072..."];

static CHI_PDF_RELATED: [&str; 2] = ["chi_cdf", "f_pdf"];

impl FunctionPlugin for ChiPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_pdf",
            description: "Chi-squared distribution PDF",
            usage: "chi_pdf(x, df)",
            args: &CHI_PDF_ARGS,
            returns: "Number",
            examples: &CHI_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &CHI_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("chi_pdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_pdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_pdf", "df", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("chi_pdf() requires df > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = chi_pdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn chi_pdf_f64(x: f64, df: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        if df < 2.0 {
            return f64::INFINITY;
        } else if df == 2.0 {
            return 0.5;
        } else {
            return 0.0;
        }
    }

    let k = df / 2.0;
    let log_pdf = -k * 2.0_f64.ln() - gamma_ln(k) + (k - 1.0) * x.ln() - x / 2.0;
    log_pdf.exp()
}

// ============ Chi CDF ============

pub struct ChiCdf;

static CHI_CDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
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

static CHI_CDF_EXAMPLES: [&str; 1] = ["chi_cdf(3.84, 1) → 0.95"];

static CHI_CDF_RELATED: [&str; 2] = ["chi_pdf", "chi_inv"];

impl FunctionPlugin for ChiCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_cdf",
            description: "Chi-squared distribution CDF",
            usage: "chi_cdf(x, df)",
            args: &CHI_CDF_ARGS,
            returns: "Number",
            examples: &CHI_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &CHI_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("chi_cdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_cdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_cdf", "df", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("chi_cdf() requires df > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = chi_cdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

pub fn chi_cdf_f64(x: f64, df: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    // Chi-squared CDF = lower regularized incomplete gamma function
    // P(k/2, x/2) where P is the regularized gamma function
    lower_incomplete_gamma(df / 2.0, x / 2.0)
}

// ============ Chi Inverse ============

pub struct ChiInv;

static CHI_INV_ARGS: [ArgMeta; 2] = [
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

static CHI_INV_EXAMPLES: [&str; 1] = ["chi_inv(0.95, 1) → 3.84"];

static CHI_INV_RELATED: [&str; 2] = ["chi_cdf", "f_inv"];

impl FunctionPlugin for ChiInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_inv",
            description: "Chi-squared distribution inverse (quantile)",
            usage: "chi_inv(p, df)",
            args: &CHI_INV_ARGS,
            returns: "Number",
            examples: &CHI_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &CHI_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("chi_inv", 2, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_inv", "p", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_inv", "df", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("chi_inv() requires 0 < p < 1"));
        }
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("chi_inv() requires df > 0"));
        }

        let result = chi_inv_f64(p_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn chi_inv_f64(p: f64, df: f64) -> f64 {
    // Newton-Raphson
    let mut x = df; // Initial guess

    for _ in 0..100 {
        let cdf = chi_cdf_f64(x, df);
        let pdf = chi_pdf_f64(x, df);
        if pdf.abs() < 1e-15 {
            break;
        }
        let dx = (cdf - p) / pdf;
        x -= dx;
        if x < 0.0 {
            x = 0.001;
        }
        if dx.abs() < 1e-12 {
            break;
        }
    }

    x
}

/// Lower regularized incomplete gamma function
fn lower_incomplete_gamma(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        // Use series representation
        gamma_series(a, x)
    } else {
        // Use continued fraction representation
        1.0 - gamma_cf(a, x)
    }
}

fn gamma_series(a: f64, x: f64) -> f64 {
    let gln = gamma_ln(a);
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut del = sum;

    for _ in 0..200 {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 3e-14 {
            break;
        }
    }

    sum * (-x + a * x.ln() - gln).exp()
}

fn gamma_cf(a: f64, x: f64) -> f64 {
    let gln = gamma_ln(a);
    let fpmin = 1e-30;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / fpmin;
    let mut d = 1.0 / b;
    let mut h = d;

    for i in 1..=200 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = b + an / c;
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

    (-x + a * x.ln() - gln).exp() * h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_chi_cdf() {
        let chi_cdf = ChiCdf;
        let args = vec![
            Value::Number(Number::from_str("3.84").unwrap()),
            Value::Number(Number::from_i64(1)),
        ];
        let result = chi_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // χ²(3.84, df=1) ≈ 0.95
        assert!((f - 0.95).abs() < 0.01);
    }
}
