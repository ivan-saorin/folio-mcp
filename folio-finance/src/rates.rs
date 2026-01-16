//! Interest rate conversion functions: effective_rate, nominal_rate, continuous_rate,
//! discount_rate, real_rate

use folio_plugin::prelude::*;
use crate::helpers::*;

// ============ Effective Rate ============

pub struct EffectiveRate;

static EFF_RATE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "nominal",
        typ: "Number",
        description: "Nominal annual rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "periods",
        typ: "Number",
        description: "Compounding periods per year",
        optional: false,
        default: None,
    },
];

static EFF_RATE_EXAMPLES: [&str; 1] = ["effective_rate(0.12, 12) → 0.1268"];

static EFF_RATE_RELATED: [&str; 2] = ["nominal_rate", "continuous_rate"];

impl FunctionPlugin for EffectiveRate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "effective_rate",
            description: "Convert nominal to effective annual rate: (1 + nominal/periods)^periods - 1",
            usage: "effective_rate(nominal, periods)",
            args: &EFF_RATE_ARGS,
            returns: "Number",
            examples: &EFF_RATE_EXAMPLES,
            category: "finance/rates",
            source: None,
            related: &EFF_RATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("effective_rate", 2, args.len()));
        }

        let nominal = match extract_number(&args[0], "effective_rate", "nominal") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let periods = match extract_number(&args[1], "effective_rate", "periods") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if periods.is_zero() {
            return Value::Error(FolioError::domain_error("effective_rate: periods must be positive"));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        // (1 + nominal/periods)^periods - 1
        let rate_per_period = match nominal.checked_div(&periods) {
            Ok(r) => r,
            Err(e) => return Value::Error(e.into()),
        };

        let base = one.add(&rate_per_period);
        let effective = compound_factor(&rate_per_period, &periods, precision).sub(&one);

        Value::Number(effective)
    }
}

// ============ Nominal Rate ============

pub struct NominalRate;

static NOM_RATE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "effective",
        typ: "Number",
        description: "Effective annual rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "periods",
        typ: "Number",
        description: "Compounding periods per year",
        optional: false,
        default: None,
    },
];

static NOM_RATE_EXAMPLES: [&str; 1] = ["nominal_rate(0.1268, 12) → 0.12"];

static NOM_RATE_RELATED: [&str; 2] = ["effective_rate", "continuous_rate"];

impl FunctionPlugin for NominalRate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nominal_rate",
            description: "Convert effective to nominal rate: periods × ((1 + effective)^(1/periods) - 1)",
            usage: "nominal_rate(effective, periods)",
            args: &NOM_RATE_ARGS,
            returns: "Number",
            examples: &NOM_RATE_EXAMPLES,
            category: "finance/rates",
            source: None,
            related: &NOM_RATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("nominal_rate", 2, args.len()));
        }

        let effective = match extract_number(&args[0], "nominal_rate", "effective") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let periods = match extract_number(&args[1], "nominal_rate", "periods") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if periods.is_zero() {
            return Value::Error(FolioError::domain_error("nominal_rate: periods must be positive"));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        // periods × ((1 + effective)^(1/periods) - 1)
        let base = one.add(&effective);
        let exponent = match one.checked_div(&periods) {
            Ok(e) => e,
            Err(e) => return Value::Error(e.into()),
        };

        let rate_per_period = base.pow_real(&exponent, precision).sub(&one);
        let nominal = periods.mul(&rate_per_period);

        Value::Number(nominal)
    }
}

// ============ Continuous Rate ============

pub struct ContinuousRate;

static CONT_RATE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "nominal",
        typ: "Number",
        description: "Nominal annual rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "periods",
        typ: "Number",
        description: "Compounding periods per year",
        optional: false,
        default: None,
    },
];

static CONT_RATE_EXAMPLES: [&str; 1] = ["continuous_rate(0.12, 12) → 0.1194"];

static CONT_RATE_RELATED: [&str; 2] = ["effective_rate", "nominal_rate"];

impl FunctionPlugin for ContinuousRate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "continuous_rate",
            description: "Convert to continuously compounded rate: periods × ln(1 + nominal/periods)",
            usage: "continuous_rate(nominal, periods)",
            args: &CONT_RATE_ARGS,
            returns: "Number",
            examples: &CONT_RATE_EXAMPLES,
            category: "finance/rates",
            source: None,
            related: &CONT_RATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("continuous_rate", 2, args.len()));
        }

        let nominal = match extract_number(&args[0], "continuous_rate", "nominal") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let periods = match extract_number(&args[1], "continuous_rate", "periods") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if periods.is_zero() {
            return Value::Error(FolioError::domain_error("continuous_rate: periods must be positive"));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        // periods × ln(1 + nominal/periods)
        let rate_per_period = match nominal.checked_div(&periods) {
            Ok(r) => r,
            Err(e) => return Value::Error(e.into()),
        };

        let ln_arg = one.add(&rate_per_period);
        let ln_val = match ln_arg.ln(precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };
        let continuous = periods.mul(&ln_val);

        Value::Number(continuous)
    }
}

// ============ Discount Rate ============

pub struct DiscountRate;

static DISC_RATE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "future_value",
        typ: "Number",
        description: "Future value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "present_value",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "periods",
        typ: "Number",
        description: "Number of periods",
        optional: false,
        default: None,
    },
];

static DISC_RATE_EXAMPLES: [&str; 1] = ["discount_rate(15000, 10000, 5) → 0.0845"];

static DISC_RATE_RELATED: [&str; 2] = ["effective_rate", "cagr"];

impl FunctionPlugin for DiscountRate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "discount_rate",
            description: "Implied discount rate: (fv/pv)^(1/periods) - 1",
            usage: "discount_rate(future_value, present_value, periods)",
            args: &DISC_RATE_ARGS,
            returns: "Number",
            examples: &DISC_RATE_EXAMPLES,
            category: "finance/rates",
            source: None,
            related: &DISC_RATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("discount_rate", 3, args.len()));
        }

        let fv = match extract_number(&args[0], "discount_rate", "future_value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[1], "discount_rate", "present_value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let periods = match extract_number(&args[2], "discount_rate", "periods") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if pv.is_zero() {
            return Value::Error(FolioError::domain_error("discount_rate: present_value must be non-zero"));
        }
        if periods.is_zero() {
            return Value::Error(FolioError::domain_error("discount_rate: periods must be positive"));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        // (fv/pv)^(1/periods) - 1
        let ratio = match fv.checked_div(&pv) {
            Ok(r) => r,
            Err(e) => return Value::Error(e.into()),
        };

        let exponent = match one.checked_div(&periods) {
            Ok(e) => e,
            Err(e) => return Value::Error(e.into()),
        };

        let rate = ratio.pow_real(&exponent, precision).sub(&one);
        Value::Number(rate)
    }
}

// ============ Real Rate (Fisher Equation) ============

pub struct RealRate;

static REAL_RATE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "nominal",
        typ: "Number",
        description: "Nominal interest rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "inflation",
        typ: "Number",
        description: "Inflation rate",
        optional: false,
        default: None,
    },
];

static REAL_RATE_EXAMPLES: [&str; 1] = ["real_rate(0.08, 0.03) → 0.0485"];

static REAL_RATE_RELATED: [&str; 2] = ["effective_rate", "nominal_rate"];

impl FunctionPlugin for RealRate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "real_rate",
            description: "Fisher equation real interest rate: (1 + nominal) / (1 + inflation) - 1",
            usage: "real_rate(nominal, inflation)",
            args: &REAL_RATE_ARGS,
            returns: "Number",
            examples: &REAL_RATE_EXAMPLES,
            category: "finance/rates",
            source: None,
            related: &REAL_RATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("real_rate", 2, args.len()));
        }

        let nominal = match extract_number(&args[0], "real_rate", "nominal") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let inflation = match extract_number(&args[1], "real_rate", "inflation") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let one = Number::from_i64(1);
        let divisor = one.add(&inflation);

        if divisor.is_zero() {
            return Value::Error(FolioError::domain_error(
                "real_rate: inflation must not equal -1",
            ));
        }

        // (1 + nominal) / (1 + inflation) - 1
        let numerator = one.add(&nominal);
        match numerator.checked_div(&divisor) {
            Ok(ratio) => Value::Number(ratio.sub(&one)),
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
    fn test_effective_rate() {
        let f = EffectiveRate;
        let args = vec![
            Value::Number(Number::from_str("0.12").unwrap()),
            Value::Number(Number::from_i64(12)),
        ];
        let result = f.call(&args, &eval_ctx());
        let rate = result.as_number().unwrap();
        // Expected: approximately 0.1268 (12.68%)
        let expected = Number::from_str("0.1268").unwrap();
        let diff = rate.sub(&expected).abs();
        assert!(diff < Number::from_str("0.001").unwrap());
    }

    #[test]
    fn test_nominal_rate() {
        let f = NominalRate;
        let args = vec![
            Value::Number(Number::from_str("0.1268").unwrap()),
            Value::Number(Number::from_i64(12)),
        ];
        let result = f.call(&args, &eval_ctx());
        let rate = result.as_number().unwrap();
        // Expected: approximately 0.12 (12%)
        let expected = Number::from_str("0.12").unwrap();
        let diff = rate.sub(&expected).abs();
        assert!(diff < Number::from_str("0.001").unwrap());
    }

    #[test]
    fn test_discount_rate() {
        let f = DiscountRate;
        let args = vec![
            Value::Number(Number::from_i64(15000)),
            Value::Number(Number::from_i64(10000)),
            Value::Number(Number::from_i64(5)),
        ];
        let result = f.call(&args, &eval_ctx());
        let rate = result.as_number().unwrap();
        // Expected: approximately 0.0845 (8.45%)
        let expected = Number::from_str("0.0845").unwrap();
        let diff = rate.sub(&expected).abs();
        assert!(diff < Number::from_str("0.001").unwrap());
    }

    #[test]
    fn test_real_rate() {
        let f = RealRate;
        let args = vec![
            Value::Number(Number::from_str("0.08").unwrap()),
            Value::Number(Number::from_str("0.03").unwrap()),
        ];
        let result = f.call(&args, &eval_ctx());
        let rate = result.as_number().unwrap();
        // Expected: approximately 0.0485 (4.85%)
        let expected = Number::from_str("0.0485").unwrap();
        let diff = rate.sub(&expected).abs();
        assert!(diff < Number::from_str("0.001").unwrap());
    }
}
