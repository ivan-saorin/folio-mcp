//! Loan calculation functions: pmt, ppmt, ipmt, nper, rate, amortization, cumipmt, cumprinc

use folio_plugin::prelude::*;
use crate::helpers::*;
use std::collections::HashMap;

// ============ PMT (Payment) ============

pub struct Pmt;

static PMT_ARGS: [ArgMeta; 5] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value (loan amount)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "fv",
        typ: "Number",
        description: "Future value",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
];

static PMT_EXAMPLES: [&str; 1] = ["pmt(0.05/12, 360, 250000) → -1342.05"];

static PMT_RELATED: [&str; 4] = ["ppmt", "ipmt", "nper", "rate"];

impl FunctionPlugin for Pmt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pmt",
            description: "Payment for a loan or annuity",
            usage: "pmt(rate, nper, pv, [fv], [type])",
            args: &PMT_ARGS,
            returns: "Number",
            examples: &PMT_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &PMT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("pmt", 3, args.len()));
        }

        let rate = match extract_number(&args[0], "pmt", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[1], "pmt", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[2], "pmt", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let fv = extract_optional_number(args, 3).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 4, 0);

        let precision = get_precision(ctx);
        match calculate_pmt(&rate, &nper, &pv, &fv, type_, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_pmt(
    rate: &Number,
    nper: &Number,
    pv: &Number,
    fv: &Number,
    type_: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    let one = Number::from_i64(1);

    if rate.is_zero() {
        // Simple case: pmt = -(pv + fv) / nper
        let sum = pv.add(fv);
        let pmt = sum.mul(&Number::from_i64(-1)).checked_div(nper)?;
        return Ok(pmt);
    }

    // (1 + rate)^nper
    let factor = compound_factor(rate, nper, precision);

    // pmt = (pv * factor + fv) * rate / ((factor - 1) * (1 + rate * type))
    let numerator = pv.mul(&factor).add(fv).mul(rate);

    let type_adj = if type_ != 0 {
        one.add(rate)
    } else {
        one.clone()
    };
    let denominator = factor.sub(&one).mul(&type_adj);

    let pmt = numerator.checked_div(&denominator)?.mul(&Number::from_i64(-1));
    Ok(pmt)
}

// ============ PPMT (Principal Payment) ============

pub struct Ppmt;

static PPMT_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "per",
        typ: "Number",
        description: "Period number (1-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Total number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "fv",
        typ: "Number",
        description: "Future value",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
];

static PPMT_EXAMPLES: [&str; 2] = [
    "ppmt(0.05/12, 1, 360, 250000) → -300.38",
    "ppmt(0.05/12, 120, 360, 250000) → -492.15",
];

static PPMT_RELATED: [&str; 3] = ["ipmt", "pmt", "cumprinc"];

impl FunctionPlugin for Ppmt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ppmt",
            description: "Principal portion of a specific payment",
            usage: "ppmt(rate, per, nper, pv, [fv], [type])",
            args: &PPMT_ARGS,
            returns: "Number",
            examples: &PPMT_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &PPMT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 4 {
            return Value::Error(FolioError::arg_count("ppmt", 4, args.len()));
        }

        let rate = match extract_number(&args[0], "ppmt", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let per = match extract_number(&args[1], "ppmt", "per") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[2], "ppmt", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[3], "ppmt", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let fv = extract_optional_number(args, 4).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 5, 0);

        let precision = get_precision(ctx);
        match calculate_ppmt(&rate, &per, &nper, &pv, &fv, type_, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_ppmt(
    rate: &Number,
    per: &Number,
    nper: &Number,
    pv: &Number,
    fv: &Number,
    type_: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    let pmt = calculate_pmt(rate, nper, pv, fv, type_, precision)?;
    let ipmt = calculate_ipmt(rate, per, nper, pv, fv, type_, precision)?;
    Ok(pmt.sub(&ipmt))
}

// ============ IPMT (Interest Payment) ============

pub struct Ipmt;

static IPMT_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "per",
        typ: "Number",
        description: "Period number (1-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Total number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "fv",
        typ: "Number",
        description: "Future value",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
];

static IPMT_EXAMPLES: [&str; 2] = [
    "ipmt(0.05/12, 1, 360, 250000) → -1041.67",
    "ipmt(0.05/12, 120, 360, 250000) → -849.90",
];

static IPMT_RELATED: [&str; 3] = ["ppmt", "pmt", "cumipmt"];

impl FunctionPlugin for Ipmt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ipmt",
            description: "Interest portion of a specific payment",
            usage: "ipmt(rate, per, nper, pv, [fv], [type])",
            args: &IPMT_ARGS,
            returns: "Number",
            examples: &IPMT_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &IPMT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 4 {
            return Value::Error(FolioError::arg_count("ipmt", 4, args.len()));
        }

        let rate = match extract_number(&args[0], "ipmt", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let per = match extract_number(&args[1], "ipmt", "per") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[2], "ipmt", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[3], "ipmt", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let fv = extract_optional_number(args, 4).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 5, 0);

        let precision = get_precision(ctx);
        match calculate_ipmt(&rate, &per, &nper, &pv, &fv, type_, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_ipmt(
    rate: &Number,
    per: &Number,
    nper: &Number,
    pv: &Number,
    fv: &Number,
    type_: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    if rate.is_zero() {
        return Ok(Number::from_i64(0));
    }

    let one = Number::from_i64(1);
    let pmt = calculate_pmt(rate, nper, pv, fv, type_, precision)?;

    // Calculate balance at start of period
    // For type=0: balance = pv * (1+r)^(per-1) + pmt * ((1+r)^(per-1) - 1) / r
    // For type=1: period 1 has no interest
    if type_ != 0 && per.to_i64() == Some(1) {
        return Ok(Number::from_i64(0));
    }

    let adj_per = if type_ != 0 {
        per.sub(&one)
    } else {
        per.sub(&one)
    };

    let factor = compound_factor(rate, &adj_per, precision);

    // Balance at start of period
    let pv_component = pv.mul(&factor);
    let pmt_factor = if adj_per.is_zero() {
        Number::from_i64(0)
    } else {
        let pmt_growth = factor.sub(&one).checked_div(rate)?;
        pmt.mul(&pmt_growth)
    };
    let balance = pv_component.add(&pmt_factor);

    // Interest = balance * rate
    let interest = balance.mul(rate).mul(&Number::from_i64(-1));
    Ok(interest)
}

// ============ NPER (Number of Periods) ============

pub struct Nper;

static NPER_ARGS: [ArgMeta; 5] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pmt",
        typ: "Number",
        description: "Payment per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "fv",
        typ: "Number",
        description: "Future value",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
];

static NPER_EXAMPLES: [&str; 1] = ["nper(0.05/12, -1500, 250000) → 294.5"];

static NPER_RELATED: [&str; 3] = ["pmt", "rate", "pv"];

impl FunctionPlugin for Nper {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nper",
            description: "Number of periods to pay off loan",
            usage: "nper(rate, pmt, pv, [fv], [type])",
            args: &NPER_ARGS,
            returns: "Number",
            examples: &NPER_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &NPER_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("nper", 3, args.len()));
        }

        let rate = match extract_number(&args[0], "nper", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pmt = match extract_number(&args[1], "nper", "pmt") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[2], "nper", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let fv = extract_optional_number(args, 3).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 4, 0);

        let precision = get_precision(ctx);
        match calculate_nper(&rate, &pmt, &pv, &fv, type_, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_nper(
    rate: &Number,
    pmt: &Number,
    pv: &Number,
    fv: &Number,
    type_: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    let one = Number::from_i64(1);

    if rate.is_zero() {
        // nper = -(pv + fv) / pmt
        if pmt.is_zero() {
            return Err(FolioError::domain_error("nper: pmt must be non-zero when rate is 0"));
        }
        let nper = pv.add(fv).mul(&Number::from_i64(-1)).checked_div(pmt)?;
        return Ok(nper);
    }

    // nper = log((pmt*(1+r*type) - fv*r) / (pmt*(1+r*type) + pv*r)) / log(1+r)
    let type_adj = if type_ != 0 {
        one.add(rate)
    } else {
        one.clone()
    };

    let pmt_adj = pmt.mul(&type_adj);
    let fv_r = fv.mul(rate);
    let pv_r = pv.mul(rate);

    let numerator = pmt_adj.sub(&fv_r);
    let denominator = pmt_adj.add(&pv_r);

    if denominator.is_zero() || numerator.is_zero() {
        return Err(FolioError::domain_error(
            "nper: Cannot compute number of periods with these parameters",
        ));
    }

    let ratio = numerator.checked_div(&denominator)?;
    if ratio.is_negative() || ratio.is_zero() {
        return Err(FolioError::domain_error(
            "nper: Parameters result in infinite or undefined periods",
        ));
    }

    let log_ratio = ratio.ln(precision)?;
    let log_base = one.add(rate).ln(precision)?;

    let nper = log_ratio.checked_div(&log_base)?;
    Ok(nper)
}

// ============ Rate ============

pub struct Rate;

static RATE_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pmt",
        typ: "Number",
        description: "Payment per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "fv",
        typ: "Number",
        description: "Future value",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "guess",
        typ: "Number",
        description: "Initial guess",
        optional: true,
        default: Some("0.1"),
    },
];

static RATE_EXAMPLES: [&str; 1] = ["rate(360, -1342.05, 250000) → 0.00417"];

static RATE_RELATED: [&str; 3] = ["pmt", "nper", "effective_rate"];

impl FunctionPlugin for Rate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "rate",
            description: "Interest rate per period using Newton-Raphson",
            usage: "rate(nper, pmt, pv, [fv], [type], [guess])",
            args: &RATE_ARGS,
            returns: "Number",
            examples: &RATE_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &RATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("rate", 3, args.len()));
        }

        let nper = match extract_number(&args[0], "rate", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pmt = match extract_number(&args[1], "rate", "pmt") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[2], "rate", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let fv = extract_optional_number(args, 3).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 4, 0);
        let guess = extract_optional_number(args, 5)
            .unwrap_or_else(|| Number::from_str("0.1").unwrap());

        let precision = get_precision(ctx);
        match calculate_rate(&nper, &pmt, &pv, &fv, type_, &guess, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_rate(
    nper: &Number,
    pmt: &Number,
    pv: &Number,
    fv: &Number,
    type_: i64,
    guess: &Number,
    precision: u32,
) -> Result<Number, FolioError> {
    let one = Number::from_i64(1);

    // Function: calculate FV given rate, should equal target fv
    let f = |rate: &Number| -> Number {
        if rate.is_zero() {
            // FV = -(pv + pmt * nper)
            let result = pv.add(&pmt.mul(nper)).add(fv);
            return result;
        }

        let factor = compound_factor(rate, nper, precision);
        let type_adj = if type_ != 0 { one.add(rate) } else { one.clone() };

        // FV = pv * factor + pmt * (factor - 1) / rate * type_adj + fv
        let pmt_part = if let Ok(p) = pmt.mul(&factor.sub(&one)).checked_div(rate) {
            p.mul(&type_adj)
        } else {
            Number::from_i64(0)
        };

        pv.mul(&factor).add(&pmt_part).add(fv)
    };

    // Derivative
    let df = |rate: &Number| -> Number {
        if rate.is_zero() {
            return Number::from_i64(0);
        }

        let factor = compound_factor(rate, nper, precision);
        let factor_deriv = nper.mul(&factor).checked_div(&one.add(rate)).unwrap_or_else(|_| Number::from_i64(0));

        // d/dr[pv * factor] = pv * factor_deriv
        let pv_deriv = pv.mul(&factor_deriv);

        // d/dr[pmt * (factor - 1) / r * type_adj] is more complex
        // Approximate with numerical derivative for robustness
        let eps = Number::from_str("0.0000001").unwrap();
        let f_plus = f(&rate.add(&eps));
        let f_minus = f(&rate.sub(&eps));
        let two_eps = eps.mul(&Number::from_i64(2));
        f_plus.sub(&f_minus).checked_div(&two_eps).unwrap_or_else(|_| Number::from_i64(1))
    };

    let tol = Number::from_str("0.000000000001").unwrap();
    match newton_raphson(guess.clone(), f, df, 100, &tol, precision) {
        Some(rate) => Ok(rate),
        None => Err(FolioError::domain_error(
            "rate: Failed to converge. Try a different initial guess.",
        )),
    }
}

// ============ Amortization Schedule ============

pub struct Amortization;

static AMORTIZATION_ARGS: [ArgMeta; 4] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value (loan amount)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "periods_to_show",
        typ: "Number",
        description: "Number of periods to include in schedule",
        optional: true,
        default: None,
    },
];

static AMORTIZATION_EXAMPLES: [&str; 1] = ["amortization(0.05/12, 360, 250000, 12)"];

static AMORTIZATION_RELATED: [&str; 3] = ["pmt", "ppmt", "ipmt"];

impl FunctionPlugin for Amortization {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "amortization",
            description: "Full amortization schedule",
            usage: "amortization(rate, nper, pv, [periods_to_show])",
            args: &AMORTIZATION_ARGS,
            returns: "Object",
            examples: &AMORTIZATION_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &AMORTIZATION_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("amortization", 3, args.len()));
        }

        let rate = match extract_number(&args[0], "amortization", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[1], "amortization", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[2], "amortization", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let nper_i64 = nper.to_i64().unwrap_or(0);
        let periods_to_show = extract_int_or_default(args, 3, nper_i64) as usize;

        let precision = get_precision(ctx);

        let pmt = match calculate_pmt(&rate, &nper, &pv, &Number::from_i64(0), 0, precision) {
            Ok(p) => p,
            Err(e) => return Value::Error(e),
        };

        let mut schedule = Vec::new();
        let mut balance = pv.clone();
        let mut total_interest = Number::from_i64(0);
        let mut total_principal = Number::from_i64(0);

        let show_count = periods_to_show.min(nper_i64 as usize);

        for period in 1..=nper_i64 as usize {
            let interest = balance.mul(&rate);
            let principal = pmt.mul(&Number::from_i64(-1)).sub(&interest);
            balance = balance.sub(&principal);

            total_interest = total_interest.add(&interest);
            total_principal = total_principal.add(&principal);

            if period <= show_count {
                let mut row = HashMap::new();
                row.insert("period".to_string(), Value::Number(Number::from_i64(period as i64)));
                row.insert("payment".to_string(), Value::Number(pmt.mul(&Number::from_i64(-1))));
                row.insert("principal".to_string(), Value::Number(principal));
                row.insert("interest".to_string(), Value::Number(interest));
                row.insert("balance".to_string(), Value::Number(balance.clone()));
                schedule.push(Value::Object(row));
            }
        }

        let mut result = HashMap::new();
        result.insert("payment".to_string(), Value::Number(pmt.mul(&Number::from_i64(-1))));
        result.insert("total_interest".to_string(), Value::Number(total_interest));
        result.insert("total_principal".to_string(), Value::Number(total_principal));
        result.insert("schedule".to_string(), Value::List(schedule));

        Value::Object(result)
    }
}

// ============ CUMIPMT (Cumulative Interest) ============

pub struct Cumipmt;

static CUMIPMT_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Total number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start_period",
        typ: "Number",
        description: "Starting period (1-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end_period",
        typ: "Number",
        description: "Ending period (1-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
];

static CUMIPMT_EXAMPLES: [&str; 1] = ["cumipmt(0.05/12, 360, 250000, 1, 12) → -12387.45"];

static CUMIPMT_RELATED: [&str; 2] = ["cumprinc", "ipmt"];

impl FunctionPlugin for Cumipmt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cumipmt",
            description: "Cumulative interest paid between periods",
            usage: "cumipmt(rate, nper, pv, start_period, end_period, [type])",
            args: &CUMIPMT_ARGS,
            returns: "Number",
            examples: &CUMIPMT_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &CUMIPMT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 5 {
            return Value::Error(FolioError::arg_count("cumipmt", 5, args.len()));
        }

        let rate = match extract_number(&args[0], "cumipmt", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[1], "cumipmt", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[2], "cumipmt", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let start = match extract_number(&args[3], "cumipmt", "start_period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let end = match extract_number(&args[4], "cumipmt", "end_period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let type_ = extract_int_or_default(args, 5, 0);

        let precision = get_precision(ctx);
        let fv = Number::from_i64(0);
        let mut total = Number::from_i64(0);

        for per in start..=end {
            let per_num = Number::from_i64(per);
            match calculate_ipmt(&rate, &per_num, &nper, &pv, &fv, type_, precision) {
                Ok(interest) => total = total.add(&interest),
                Err(e) => return Value::Error(e),
            }
        }

        Value::Number(total)
    }
}

// ============ CUMPRINC (Cumulative Principal) ============

pub struct Cumprinc;

static CUMPRINC_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Interest rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "nper",
        typ: "Number",
        description: "Total number of periods",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pv",
        typ: "Number",
        description: "Present value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start_period",
        typ: "Number",
        description: "Starting period (1-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end_period",
        typ: "Number",
        description: "Ending period (1-based)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "type",
        typ: "Number",
        description: "0 = end of period, 1 = beginning",
        optional: true,
        default: Some("0"),
    },
];

static CUMPRINC_EXAMPLES: [&str; 1] = ["cumprinc(0.05/12, 360, 250000, 1, 12) → -3717.15"];

static CUMPRINC_RELATED: [&str; 2] = ["cumipmt", "ppmt"];

impl FunctionPlugin for Cumprinc {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cumprinc",
            description: "Cumulative principal paid between periods",
            usage: "cumprinc(rate, nper, pv, start_period, end_period, [type])",
            args: &CUMPRINC_ARGS,
            returns: "Number",
            examples: &CUMPRINC_EXAMPLES,
            category: "finance/loans",
            source: None,
            related: &CUMPRINC_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 5 {
            return Value::Error(FolioError::arg_count("cumprinc", 5, args.len()));
        }

        let rate = match extract_number(&args[0], "cumprinc", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[1], "cumprinc", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = match extract_number(&args[2], "cumprinc", "pv") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let start = match extract_number(&args[3], "cumprinc", "start_period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let end = match extract_number(&args[4], "cumprinc", "end_period") {
            Ok(n) => n.to_i64().unwrap_or(1),
            Err(e) => return Value::Error(e),
        };
        let type_ = extract_int_or_default(args, 5, 0);

        let precision = get_precision(ctx);
        let fv = Number::from_i64(0);
        let mut total = Number::from_i64(0);

        for per in start..=end {
            let per_num = Number::from_i64(per);
            match calculate_ppmt(&rate, &per_num, &nper, &pv, &fv, type_, precision) {
                Ok(principal) => total = total.add(&principal),
                Err(e) => return Value::Error(e),
            }
        }

        Value::Number(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_pmt() {
        let f = Pmt;
        let rate = Number::from_str("0.05").unwrap()
            .checked_div(&Number::from_i64(12)).unwrap();
        let args = vec![
            Value::Number(rate),
            Value::Number(Number::from_i64(360)),
            Value::Number(Number::from_i64(250000)),
        ];
        let result = f.call(&args, &eval_ctx());
        let pmt = result.as_number().unwrap();
        // Expected: approximately -1342.05
        assert!(pmt.is_negative());
        assert!(pmt.abs() > Number::from_i64(1300));
        assert!(pmt.abs() < Number::from_i64(1400));
    }

    #[test]
    fn test_ppmt_and_ipmt_sum_to_pmt() {
        let rate = Number::from_str("0.05").unwrap()
            .checked_div(&Number::from_i64(12)).unwrap();
        let nper = Number::from_i64(360);
        let pv = Number::from_i64(250000);
        let fv = Number::from_i64(0);

        let pmt = calculate_pmt(&rate, &nper, &pv, &fv, 0, 50).unwrap();

        for per in [1, 10, 50, 100, 200, 359, 360] {
            let per_num = Number::from_i64(per);
            let ppmt = calculate_ppmt(&rate, &per_num, &nper, &pv, &fv, 0, 50).unwrap();
            let ipmt = calculate_ipmt(&rate, &per_num, &nper, &pv, &fv, 0, 50).unwrap();

            let sum = ppmt.add(&ipmt);
            let diff = sum.sub(&pmt).abs();
            assert!(diff < Number::from_str("0.01").unwrap(), "Period {}: ppmt + ipmt != pmt", per);
        }
    }

    #[test]
    fn test_nper() {
        let f = Nper;
        let rate = Number::from_str("0.05").unwrap()
            .checked_div(&Number::from_i64(12)).unwrap();
        let args = vec![
            Value::Number(rate),
            Value::Number(Number::from_i64(-1500)),
            Value::Number(Number::from_i64(250000)),
        ];
        let result = f.call(&args, &eval_ctx());
        let nper = result.as_number().unwrap();
        // For $250k loan at 5% APR with $1500/month payment
        // Result is a positive number of periods
        assert!(nper > &Number::from_i64(0));
        assert!(nper < &Number::from_i64(500));
    }
}
