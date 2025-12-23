//! F distribution functions

use folio_plugin::prelude::*;
use super::t::{gamma_ln, regularized_incomplete_beta};

// ============ F PDF ============

pub struct FPdf;

static F_PDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df1",
        typ: "Number",
        description: "Numerator degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df2",
        typ: "Number",
        description: "Denominator degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static F_PDF_EXAMPLES: [&str; 1] = ["f_pdf(2, 5, 10) → 0.127..."];

static F_PDF_RELATED: [&str; 2] = ["f_cdf", "chi_pdf"];

impl FunctionPlugin for FPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_pdf",
            description: "F-distribution PDF",
            usage: "f_pdf(x, df1, df2)",
            args: &F_PDF_ARGS,
            returns: "Number",
            examples: &F_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &F_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("f_pdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_pdf", "x", "Number", other.type_name())),
        };

        let df1 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_pdf", "df1", "Number", other.type_name())),
        };

        let df2 = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_pdf", "df2", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df1_f64 = df1.to_f64().unwrap_or(1.0);
        let df2_f64 = df2.to_f64().unwrap_or(1.0);

        if df1_f64 <= 0.0 || df2_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("f_pdf() requires df1 > 0 and df2 > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = f_pdf_f64(x_f64, df1_f64, df2_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn f_pdf_f64(x: f64, d1: f64, d2: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        if d1 < 2.0 {
            return f64::INFINITY;
        } else if d1 == 2.0 {
            return 1.0;
        } else {
            return 0.0;
        }
    }

    let log_num = (d1 / 2.0) * (d1).ln() + (d2 / 2.0) * (d2).ln()
        + ((d1 / 2.0) - 1.0) * x.ln();
    let log_den = gamma_ln(d1 / 2.0) + gamma_ln(d2 / 2.0)
        - gamma_ln((d1 + d2) / 2.0)
        + ((d1 + d2) / 2.0) * (d1 * x + d2).ln();

    (log_num - log_den).exp()
}

// ============ F CDF ============

pub struct FCdf;

static F_CDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df1",
        typ: "Number",
        description: "Numerator degrees of freedom",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df2",
        typ: "Number",
        description: "Denominator degrees of freedom",
        optional: false,
        default: None,
    },
];

static F_CDF_EXAMPLES: [&str; 1] = ["f_cdf(3.89, 3, 20) → 0.975"];

static F_CDF_RELATED: [&str; 2] = ["f_pdf", "f_inv"];

impl FunctionPlugin for FCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_cdf",
            description: "F-distribution CDF",
            usage: "f_cdf(x, df1, df2)",
            args: &F_CDF_ARGS,
            returns: "Number",
            examples: &F_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &F_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("f_cdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_cdf", "x", "Number", other.type_name())),
        };

        let df1 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_cdf", "df1", "Number", other.type_name())),
        };

        let df2 = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_cdf", "df2", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df1_f64 = df1.to_f64().unwrap_or(1.0);
        let df2_f64 = df2.to_f64().unwrap_or(1.0);

        if df1_f64 <= 0.0 || df2_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("f_cdf() requires df1 > 0 and df2 > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = f_cdf_f64(x_f64, df1_f64, df2_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

pub fn f_cdf_f64(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    // F CDF = I_{d1*x/(d1*x+d2)}(d1/2, d2/2)
    let z = d1 * x / (d1 * x + d2);
    regularized_incomplete_beta(d1 / 2.0, d2 / 2.0, z)
}

// ============ F Inverse ============

pub struct FInv;

static F_INV_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df1",
        typ: "Number",
        description: "Numerator degrees of freedom",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df2",
        typ: "Number",
        description: "Denominator degrees of freedom",
        optional: false,
        default: None,
    },
];

static F_INV_EXAMPLES: [&str; 1] = ["f_inv(0.95, 5, 10) → 3.33"];

static F_INV_RELATED: [&str; 2] = ["f_cdf", "chi_inv"];

impl FunctionPlugin for FInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_inv",
            description: "F-distribution inverse (quantile)",
            usage: "f_inv(p, df1, df2)",
            args: &F_INV_ARGS,
            returns: "Number",
            examples: &F_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &F_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("f_inv", 3, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_inv", "p", "Number", other.type_name())),
        };

        let df1 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_inv", "df1", "Number", other.type_name())),
        };

        let df2 = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_inv", "df2", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        let df1_f64 = df1.to_f64().unwrap_or(1.0);
        let df2_f64 = df2.to_f64().unwrap_or(1.0);

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("f_inv() requires 0 < p < 1"));
        }
        if df1_f64 <= 0.0 || df2_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("f_inv() requires df1 > 0 and df2 > 0"));
        }

        let result = f_inv_f64(p_f64, df1_f64, df2_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn f_inv_f64(p: f64, d1: f64, d2: f64) -> f64 {
    // Newton-Raphson
    let mut x = 1.0; // Initial guess

    for _ in 0..100 {
        let cdf = f_cdf_f64(x, d1, d2);
        let pdf = f_pdf_f64(x, d1, d2);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_f_cdf() {
        let f_cdf = FCdf;
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(10)),
        ];
        let result = f_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // F(1, 5, 10) should be around 0.5
        assert!(f > 0.4 && f < 0.6);
    }
}
