//! Discrete distribution functions: binomial, Poisson

use folio_plugin::prelude::*;
use super::t::gamma_ln;

// ============ Binomial PMF ============

pub struct BinomPmf;

static BINOM_PMF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of successes",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of trials",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability of success (0 ≤ p ≤ 1)",
        optional: false,
        default: None,
    },
];

static BINOM_PMF_EXAMPLES: [&str; 1] = ["binom_pmf(3, 10, 0.5) → 0.117..."];

static BINOM_PMF_RELATED: [&str; 2] = ["binom_cdf", "poisson_pmf"];

impl FunctionPlugin for BinomPmf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "binom_pmf",
            description: "Binomial probability mass function",
            usage: "binom_pmf(k, n, p)",
            args: &BINOM_PMF_ARGS,
            returns: "Number",
            examples: &BINOM_PMF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &BINOM_PMF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("binom_pmf", 3, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_pmf", "k", "Number", other.type_name())),
        };

        let n = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_pmf", "n", "Number", other.type_name())),
        };

        let p = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_pmf", "p", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let n_i64 = n.to_i64().unwrap_or(-1);
        let p_f64 = p.to_f64().unwrap_or(-1.0);

        if n_i64 < 0 || k_i64 < 0 {
            return Value::Error(FolioError::domain_error("binom_pmf() requires k ≥ 0 and n ≥ 0"));
        }
        if k_i64 > n_i64 {
            return Value::Number(Number::from_i64(0));
        }
        if p_f64 < 0.0 || p_f64 > 1.0 {
            return Value::Error(FolioError::domain_error("binom_pmf() requires 0 ≤ p ≤ 1"));
        }

        let result = binom_pmf_f64(k_i64 as u64, n_i64 as u64, p_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn binom_pmf_f64(k: u64, n: u64, p: f64) -> f64 {
    if k > n {
        return 0.0;
    }
    if p == 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    if p == 1.0 {
        return if k == n { 1.0 } else { 0.0 };
    }

    // Use log for numerical stability
    // PMF = C(n,k) * p^k * (1-p)^(n-k)
    // log(PMF) = log(C(n,k)) + k*log(p) + (n-k)*log(1-p)
    let log_coef = log_binomial(n, k);
    let log_prob = (k as f64) * p.ln() + ((n - k) as f64) * (1.0 - p).ln();
    (log_coef + log_prob).exp()
}

fn log_binomial(n: u64, k: u64) -> f64 {
    // log(C(n,k)) = log(n!) - log(k!) - log((n-k)!)
    // Using gamma_ln(x+1) = log(x!)
    gamma_ln((n + 1) as f64) - gamma_ln((k + 1) as f64) - gamma_ln((n - k + 1) as f64)
}

// ============ Binomial CDF ============

pub struct BinomCdf;

static BINOM_CDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of successes",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of trials",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability of success (0 ≤ p ≤ 1)",
        optional: false,
        default: None,
    },
];

static BINOM_CDF_EXAMPLES: [&str; 1] = ["binom_cdf(5, 10, 0.5) → 0.623..."];

static BINOM_CDF_RELATED: [&str; 2] = ["binom_pmf", "poisson_cdf"];

impl FunctionPlugin for BinomCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "binom_cdf",
            description: "Binomial cumulative distribution function",
            usage: "binom_cdf(k, n, p)",
            args: &BINOM_CDF_ARGS,
            returns: "Number",
            examples: &BINOM_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &BINOM_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("binom_cdf", 3, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_cdf", "k", "Number", other.type_name())),
        };

        let n = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_cdf", "n", "Number", other.type_name())),
        };

        let p = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_cdf", "p", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let n_i64 = n.to_i64().unwrap_or(-1);
        let p_f64 = p.to_f64().unwrap_or(-1.0);

        if n_i64 < 0 {
            return Value::Error(FolioError::domain_error("binom_cdf() requires n ≥ 0"));
        }
        if k_i64 < 0 {
            return Value::Number(Number::from_i64(0));
        }
        if k_i64 >= n_i64 {
            return Value::Number(Number::from_i64(1));
        }
        if p_f64 < 0.0 || p_f64 > 1.0 {
            return Value::Error(FolioError::domain_error("binom_cdf() requires 0 ≤ p ≤ 1"));
        }

        let mut cdf = 0.0;
        for i in 0..=(k_i64 as u64) {
            cdf += binom_pmf_f64(i, n_i64 as u64, p_f64);
        }

        Value::Number(Number::from_str(&format!("{:.15}", cdf)).unwrap_or(Number::from_i64(0)))
    }
}

// ============ Poisson PMF ============

pub struct PoissonPmf;

static POISSON_PMF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of events",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "λ",
        typ: "Number",
        description: "Rate parameter (must be > 0)",
        optional: false,
        default: None,
    },
];

static POISSON_PMF_EXAMPLES: [&str; 1] = ["poisson_pmf(3, 2.5) → 0.214..."];

static POISSON_PMF_RELATED: [&str; 2] = ["poisson_cdf", "binom_pmf"];

impl FunctionPlugin for PoissonPmf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "poisson_pmf",
            description: "Poisson probability mass function",
            usage: "poisson_pmf(k, λ)",
            args: &POISSON_PMF_ARGS,
            returns: "Number",
            examples: &POISSON_PMF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &POISSON_PMF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("poisson_pmf", 2, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_pmf", "k", "Number", other.type_name())),
        };

        let lambda = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_pmf", "λ", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let lambda_f64 = lambda.to_f64().unwrap_or(-1.0);

        if k_i64 < 0 {
            return Value::Number(Number::from_i64(0));
        }
        if lambda_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("poisson_pmf() requires λ > 0"));
        }

        let result = poisson_pmf_f64(k_i64 as u64, lambda_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn poisson_pmf_f64(k: u64, lambda: f64) -> f64 {
    // PMF = λ^k * e^(-λ) / k!
    // log(PMF) = k*log(λ) - λ - log(k!)
    let log_prob = (k as f64) * lambda.ln() - lambda - gamma_ln((k + 1) as f64);
    log_prob.exp()
}

// ============ Poisson CDF ============

pub struct PoissonCdf;

static POISSON_CDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of events",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "λ",
        typ: "Number",
        description: "Rate parameter (must be > 0)",
        optional: false,
        default: None,
    },
];

static POISSON_CDF_EXAMPLES: [&str; 1] = ["poisson_cdf(5, 3) → 0.916..."];

static POISSON_CDF_RELATED: [&str; 2] = ["poisson_pmf", "binom_cdf"];

impl FunctionPlugin for PoissonCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "poisson_cdf",
            description: "Poisson cumulative distribution function",
            usage: "poisson_cdf(k, λ)",
            args: &POISSON_CDF_ARGS,
            returns: "Number",
            examples: &POISSON_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &POISSON_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("poisson_cdf", 2, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_cdf", "k", "Number", other.type_name())),
        };

        let lambda = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_cdf", "λ", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let lambda_f64 = lambda.to_f64().unwrap_or(-1.0);

        if k_i64 < 0 {
            return Value::Number(Number::from_i64(0));
        }
        if lambda_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("poisson_cdf() requires λ > 0"));
        }

        let mut cdf = 0.0;
        for i in 0..=(k_i64 as u64) {
            cdf += poisson_pmf_f64(i, lambda_f64);
        }

        Value::Number(Number::from_str(&format!("{:.15}", cdf)).unwrap_or(Number::from_i64(0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_binom_pmf() {
        let binom_pmf = BinomPmf;
        let args = vec![
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_str("0.5").unwrap()),
        ];
        let result = binom_pmf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // P(X=5|n=10, p=0.5) ≈ 0.246
        assert!((f - 0.246).abs() < 0.01);
    }

    #[test]
    fn test_poisson_pmf() {
        let poisson_pmf = PoissonPmf;
        let args = vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_str("2.5").unwrap()),
        ];
        let result = poisson_pmf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // P(X=3|λ=2.5) ≈ 0.214
        assert!((f - 0.214).abs() < 0.01);
    }
}
