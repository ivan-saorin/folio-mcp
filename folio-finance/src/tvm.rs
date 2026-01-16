//! Time Value of Money functions: pv, fv, npv, xnpv, irr, xirr, mirr

use folio_plugin::prelude::*;
use crate::helpers::*;

// ============ PV (Present Value) ============

pub struct Pv;

static PV_ARGS: [ArgMeta; 5] = [
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
        name: "pmt",
        typ: "Number",
        description: "Payment per period (negative = outflow)",
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

static PV_EXAMPLES: [&str; 2] = [
    "pv(0.05, 10, -1000) → 7721.73",
    "pv(0.05, 10, -1000, 0, 1) → 8107.82",
];

static PV_RELATED: [&str; 3] = ["fv", "npv", "pmt"];

impl FunctionPlugin for Pv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pv",
            description: "Present value of an annuity",
            usage: "pv(rate, nper, pmt, [fv], [type])",
            args: &PV_ARGS,
            returns: "Number",
            examples: &PV_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &PV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("pv", 3, args.len()));
        }

        let rate = match extract_number(&args[0], "pv", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[1], "pv", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pmt = match extract_number(&args[2], "pv", "pmt") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let fv = extract_optional_number(args, 3).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 4, 0);

        let precision = get_precision(ctx);
        match calculate_pv(&rate, &nper, &pmt, &fv, type_, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_pv(
    rate: &Number,
    nper: &Number,
    pmt: &Number,
    fv: &Number,
    type_: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    if let Err(e) = validate_rate(rate, "pv") {
        return Err(e);
    }

    let one = Number::from_i64(1);

    if rate.is_zero() {
        // Simple case: no interest
        let pmt_total = pmt.mul(nper);
        let result = pmt_total.add(fv).mul(&Number::from_i64(-1));
        return Ok(result);
    }

    // (1 + rate)^nper
    let factor = compound_factor(rate, nper, precision);

    // PV of payments: pmt * (1 - (1+r)^-n) / r * (1 + r*type)
    let one_minus_factor_inv = one.sub(&one.checked_div(&factor)?);
    let pv_pmt = pmt.mul(&one_minus_factor_inv).checked_div(rate)?;

    // Adjust for beginning of period
    let pv_pmt = if type_ != 0 {
        pv_pmt.mul(&one.add(rate))
    } else {
        pv_pmt
    };

    // PV of future value
    let pv_fv = fv.checked_div(&factor)?;

    // Total PV (negate because cash flows are from perspective of investor)
    let result = pv_pmt.add(&pv_fv).mul(&Number::from_i64(-1));
    Ok(result)
}

// ============ FV (Future Value) ============

pub struct Fv;

static FV_ARGS: [ArgMeta; 5] = [
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

static FV_EXAMPLES: [&str; 1] = ["fv(0.06/12, 240, -500, 0, 0) → 231020.50"];

static FV_RELATED: [&str; 3] = ["pv", "npv", "pmt"];

impl FunctionPlugin for Fv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "fv",
            description: "Future value of an annuity",
            usage: "fv(rate, nper, pmt, [pv], [type])",
            args: &FV_ARGS,
            returns: "Number",
            examples: &FV_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &FV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("fv", 3, args.len()));
        }

        let rate = match extract_number(&args[0], "fv", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let nper = match extract_number(&args[1], "fv", "nper") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pmt = match extract_number(&args[2], "fv", "pmt") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let pv = extract_optional_number(args, 3).unwrap_or_else(|| Number::from_i64(0));
        let type_ = extract_int_or_default(args, 4, 0);

        let precision = get_precision(ctx);
        match calculate_fv(&rate, &nper, &pmt, &pv, type_, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_fv(
    rate: &Number,
    nper: &Number,
    pmt: &Number,
    pv: &Number,
    type_: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    if let Err(e) = validate_rate(rate, "fv") {
        return Err(e);
    }

    let one = Number::from_i64(1);

    if rate.is_zero() {
        // Simple case: no interest
        let pmt_total = pmt.mul(nper);
        let result = pv.add(&pmt_total).mul(&Number::from_i64(-1));
        return Ok(result);
    }

    // (1 + rate)^nper
    let factor = compound_factor(rate, nper, precision);

    // FV of present value
    let fv_pv = pv.mul(&factor);

    // FV of payments: pmt * ((1+r)^n - 1) / r * (1 + r*type)
    let factor_minus_one = factor.sub(&one);
    let fv_pmt = pmt.mul(&factor_minus_one).checked_div(rate)?;

    // Adjust for beginning of period
    let fv_pmt = if type_ != 0 {
        fv_pmt.mul(&one.add(rate))
    } else {
        fv_pmt
    };

    // Total FV (negate)
    let result = fv_pv.add(&fv_pmt).mul(&Number::from_i64(-1));
    Ok(result)
}

// ============ NPV (Net Present Value) ============

pub struct Npv;

static NPV_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Discount rate per period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "cash_flows",
        typ: "List<Number>",
        description: "Cash flows starting at period 1",
        optional: false,
        default: None,
    },
];

static NPV_EXAMPLES: [&str; 1] = ["npv(0.10, [-100000, 30000, 40000, 50000, 30000]) → 17090.42"];

static NPV_RELATED: [&str; 3] = ["xnpv", "irr", "pv"];

impl FunctionPlugin for Npv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "npv",
            description: "Net present value of cash flows (starting at period 1)",
            usage: "npv(rate, cash_flows)",
            args: &NPV_ARGS,
            returns: "Number",
            examples: &NPV_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &NPV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("npv", 2, args.len()));
        }

        let rate = match extract_number(&args[0], "npv", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let cash_flows = match extract_numbers_from_list(&args[1], "npv", "cash_flows") {
            Ok(cf) => cf,
            Err(e) => return Value::Error(e),
        };

        let precision = get_precision(ctx);
        match calculate_npv(&rate, &cash_flows, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_npv(rate: &Number, cash_flows: &[Number], precision: u32) -> Result<Number, FolioError> {
    if let Err(e) = validate_rate(rate, "npv") {
        return Err(e);
    }

    let one = Number::from_i64(1);
    let discount = one.add(rate);
    let mut npv = Number::from_i64(0);

    for (i, cf) in cash_flows.iter().enumerate() {
        let period = Number::from_i64((i + 1) as i64);
        let factor = compound_factor(&discount.sub(&one), &period, precision);
        let pv = cf.checked_div(&factor)?;
        npv = npv.add(&pv);
    }

    Ok(npv)
}

// ============ XNPV (NPV with dates) ============

pub struct Xnpv;

static XNPV_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual discount rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "cash_flows",
        typ: "List<Number>",
        description: "Cash flows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "dates",
        typ: "List<DateTime>",
        description: "Dates for each cash flow",
        optional: false,
        default: None,
    },
];

static XNPV_EXAMPLES: [&str; 1] = ["xnpv(0.08, flows, dates) → NPV with specific dates"];

static XNPV_RELATED: [&str; 3] = ["npv", "xirr", "pv"];

impl FunctionPlugin for Xnpv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "xnpv",
            description: "Net present value with specific dates (ACT/365)",
            usage: "xnpv(rate, cash_flows, dates)",
            args: &XNPV_ARGS,
            returns: "Number",
            examples: &XNPV_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &XNPV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("xnpv", 3, args.len()));
        }

        let rate = match extract_number(&args[0], "xnpv", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let cash_flows = match extract_numbers_from_list(&args[1], "xnpv", "cash_flows") {
            Ok(cf) => cf,
            Err(e) => return Value::Error(e),
        };
        let dates = match extract_dates_from_list(&args[2], "xnpv", "dates") {
            Ok(d) => d,
            Err(e) => return Value::Error(e),
        };

        if cash_flows.len() != dates.len() {
            return Value::Error(FolioError::domain_error(format!(
                "xnpv: cash_flows length ({}) must equal dates length ({})",
                cash_flows.len(),
                dates.len()
            )));
        }

        let precision = get_precision(ctx);
        match calculate_xnpv(&rate, &cash_flows, &dates, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn extract_dates_from_list(
    value: &Value,
    func: &str,
    arg: &str,
) -> Result<Vec<FolioDateTime>, FolioError> {
    match value {
        Value::List(items) => {
            let mut dates = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::DateTime(dt) => dates.push(dt.clone()),
                    Value::Error(e) => return Err(e.clone()),
                    other => {
                        return Err(FolioError::type_error(
                            "DateTime",
                            &format!("{}[{}]: {}", arg, i, other.type_name()),
                        ))
                    }
                }
            }
            Ok(dates)
        }
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type(func, arg, "List<DateTime>", other.type_name())),
    }
}

fn calculate_xnpv(
    rate: &Number,
    cash_flows: &[Number],
    dates: &[FolioDateTime],
    precision: u32,
) -> Result<Number, FolioError> {
    if cash_flows.is_empty() {
        return Ok(Number::from_i64(0));
    }

    let first_date = &dates[0];
    let one = Number::from_i64(1);
    let discount_base = one.add(rate);
    let mut npv = Number::from_i64(0);

    for (cf, date) in cash_flows.iter().zip(dates.iter()) {
        let year_frac = year_fraction_act365(first_date, date);
        let factor = discount_base.pow_real(&year_frac, precision);
        let pv = cf.checked_div(&factor)?;
        npv = npv.add(&pv);
    }

    Ok(npv)
}

// ============ IRR (Internal Rate of Return) ============

pub struct Irr;

static IRR_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "cash_flows",
        typ: "List<Number>",
        description: "Cash flows (first is typically negative investment)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "guess",
        typ: "Number",
        description: "Initial guess for rate",
        optional: true,
        default: Some("0.1"),
    },
];

static IRR_EXAMPLES: [&str; 1] = ["irr([-100000, 30000, 40000, 50000, 30000]) → 0.1567"];

static IRR_RELATED: [&str; 3] = ["xirr", "npv", "mirr"];

impl FunctionPlugin for Irr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "irr",
            description: "Internal rate of return using Newton-Raphson",
            usage: "irr(cash_flows, [guess])",
            args: &IRR_ARGS,
            returns: "Number",
            examples: &IRR_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &IRR_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("irr", 1, 0));
        }

        let cash_flows = match extract_numbers_from_list(&args[0], "irr", "cash_flows") {
            Ok(cf) => cf,
            Err(e) => return Value::Error(e),
        };
        let guess = extract_optional_number(args, 1)
            .unwrap_or_else(|| Number::from_str("0.1").unwrap());

        let precision = get_precision(ctx);
        match calculate_irr(&cash_flows, &guess, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_irr(cash_flows: &[Number], guess: &Number, precision: u32) -> Result<Number, FolioError> {
    if cash_flows.len() < 2 {
        return Err(FolioError::domain_error("irr: At least 2 cash flows required"));
    }

    // NPV function
    let npv_f = |rate: &Number| -> Number {
        let one = Number::from_i64(1);
        let mut npv = Number::from_i64(0);
        for (i, cf) in cash_flows.iter().enumerate() {
            let period = Number::from_i64(i as i64);
            let factor = compound_factor(rate, &period, precision);
            if let Ok(pv) = cf.checked_div(&factor) {
                npv = npv.add(&pv);
            }
        }
        npv
    };

    // Derivative of NPV
    let npv_df = |rate: &Number| -> Number {
        let one = Number::from_i64(1);
        let mut dnpv = Number::from_i64(0);
        for (i, cf) in cash_flows.iter().enumerate() {
            if i == 0 {
                continue; // First term has no rate dependency
            }
            let period = Number::from_i64(i as i64);
            let n_plus_1 = period.add(&one);
            let factor = compound_factor(rate, &n_plus_1, precision);
            if let Ok(term) = cf.mul(&period).mul(&Number::from_i64(-1)).checked_div(&factor) {
                dnpv = dnpv.add(&term);
            }
        }
        dnpv
    };

    let tol = Number::from_str("0.000000000001").unwrap(); // 1e-12
    match newton_raphson(guess.clone(), npv_f, npv_df, 100, &tol, precision) {
        Some(rate) => Ok(rate),
        None => Err(FolioError::domain_error(
            "irr: Failed to converge. Try a different initial guess.",
        )),
    }
}

// ============ XIRR (IRR with dates) ============

pub struct Xirr;

static XIRR_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "cash_flows",
        typ: "List<Number>",
        description: "Cash flows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "dates",
        typ: "List<DateTime>",
        description: "Dates for each cash flow",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "guess",
        typ: "Number",
        description: "Initial guess for rate",
        optional: true,
        default: Some("0.1"),
    },
];

static XIRR_EXAMPLES: [&str; 1] = ["xirr(flows, dates) → 0.1823"];

static XIRR_RELATED: [&str; 3] = ["irr", "xnpv", "mirr"];

impl FunctionPlugin for Xirr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "xirr",
            description: "Internal rate of return with specific dates",
            usage: "xirr(cash_flows, dates, [guess])",
            args: &XIRR_ARGS,
            returns: "Number",
            examples: &XIRR_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &XIRR_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("xirr", 2, args.len()));
        }

        let cash_flows = match extract_numbers_from_list(&args[0], "xirr", "cash_flows") {
            Ok(cf) => cf,
            Err(e) => return Value::Error(e),
        };
        let dates = match extract_dates_from_list(&args[1], "xirr", "dates") {
            Ok(d) => d,
            Err(e) => return Value::Error(e),
        };
        let guess = extract_optional_number(args, 2)
            .unwrap_or_else(|| Number::from_str("0.1").unwrap());

        if cash_flows.len() != dates.len() {
            return Value::Error(FolioError::domain_error(format!(
                "xirr: cash_flows length ({}) must equal dates length ({})",
                cash_flows.len(),
                dates.len()
            )));
        }

        let precision = get_precision(ctx);
        match calculate_xirr(&cash_flows, &dates, &guess, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_xirr(
    cash_flows: &[Number],
    dates: &[FolioDateTime],
    guess: &Number,
    precision: u32,
) -> Result<Number, FolioError> {
    if cash_flows.len() < 2 {
        return Err(FolioError::domain_error("xirr: At least 2 cash flows required"));
    }

    let first_date = dates[0].clone();

    // XNPV function
    let xnpv_f = |rate: &Number| -> Number {
        let one = Number::from_i64(1);
        let discount_base = one.add(rate);
        let mut npv = Number::from_i64(0);
        for (cf, date) in cash_flows.iter().zip(dates.iter()) {
            let year_frac = year_fraction_act365(&first_date, date);
            let factor = discount_base.pow_real(&year_frac, precision);
            if let Ok(pv) = cf.checked_div(&factor) {
                npv = npv.add(&pv);
            }
        }
        npv
    };

    // Derivative of XNPV
    let xnpv_df = |rate: &Number| -> Number {
        let one = Number::from_i64(1);
        let discount_base = one.add(rate);
        let mut dnpv = Number::from_i64(0);
        for (cf, date) in cash_flows.iter().zip(dates.iter()) {
            let year_frac = year_fraction_act365(&first_date, date);
            if year_frac.is_zero() {
                continue;
            }
            let factor = discount_base.pow_real(&year_frac.add(&one), precision);
            if let Ok(term) = cf.mul(&year_frac).mul(&Number::from_i64(-1)).checked_div(&factor) {
                dnpv = dnpv.add(&term);
            }
        }
        dnpv
    };

    let tol = Number::from_str("0.000000000001").unwrap();
    match newton_raphson(guess.clone(), xnpv_f, xnpv_df, 100, &tol, precision) {
        Some(rate) => Ok(rate),
        None => Err(FolioError::domain_error(
            "xirr: Failed to converge. Try a different initial guess.",
        )),
    }
}

// ============ MIRR (Modified IRR) ============

pub struct Mirr;

static MIRR_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "cash_flows",
        typ: "List<Number>",
        description: "Cash flows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "finance_rate",
        typ: "Number",
        description: "Rate paid on negative cash flows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "reinvest_rate",
        typ: "Number",
        description: "Rate earned on positive cash flows",
        optional: false,
        default: None,
    },
];

static MIRR_EXAMPLES: [&str; 1] = ["mirr(flows, 0.10, 0.12) → 0.1345"];

static MIRR_RELATED: [&str; 2] = ["irr", "npv"];

impl FunctionPlugin for Mirr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mirr",
            description: "Modified IRR (separates cost of capital from reinvestment rate)",
            usage: "mirr(cash_flows, finance_rate, reinvest_rate)",
            args: &MIRR_ARGS,
            returns: "Number",
            examples: &MIRR_EXAMPLES,
            category: "finance/tvm",
            source: None,
            related: &MIRR_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("mirr", 3, args.len()));
        }

        let cash_flows = match extract_numbers_from_list(&args[0], "mirr", "cash_flows") {
            Ok(cf) => cf,
            Err(e) => return Value::Error(e),
        };
        let finance_rate = match extract_number(&args[1], "mirr", "finance_rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let reinvest_rate = match extract_number(&args[2], "mirr", "reinvest_rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let precision = get_precision(ctx);
        match calculate_mirr(&cash_flows, &finance_rate, &reinvest_rate, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_mirr(
    cash_flows: &[Number],
    finance_rate: &Number,
    reinvest_rate: &Number,
    precision: u32,
) -> Result<Number, FolioError> {
    if cash_flows.len() < 2 {
        return Err(FolioError::domain_error("mirr: At least 2 cash flows required"));
    }

    let n = cash_flows.len() as i64;
    let one = Number::from_i64(1);

    // PV of negative cash flows (using finance_rate)
    let mut pv_neg = Number::from_i64(0);
    for (i, cf) in cash_flows.iter().enumerate() {
        if cf.is_negative() {
            let period = Number::from_i64(i as i64);
            let factor = compound_factor(finance_rate, &period, precision);
            let pv = cf.checked_div(&factor)?;
            pv_neg = pv_neg.add(&pv);
        }
    }

    // FV of positive cash flows (using reinvest_rate)
    let mut fv_pos = Number::from_i64(0);
    for (i, cf) in cash_flows.iter().enumerate() {
        if !cf.is_negative() && !cf.is_zero() {
            let periods_to_end = Number::from_i64((n - 1 - i as i64) as i64);
            let factor = compound_factor(reinvest_rate, &periods_to_end, precision);
            let fv = cf.mul(&factor);
            fv_pos = fv_pos.add(&fv);
        }
    }

    // MIRR = (FV_pos / -PV_neg)^(1/(n-1)) - 1
    if pv_neg.is_zero() {
        return Err(FolioError::domain_error("mirr: No negative cash flows found"));
    }

    let ratio = fv_pos.checked_div(&pv_neg.mul(&Number::from_i64(-1)))?;
    let n_minus_1 = Number::from_i64(n - 1);
    let exponent = one.checked_div(&n_minus_1)?;
    let mirr = ratio.pow_real(&exponent, precision).sub(&one);

    Ok(mirr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_pv_basic() {
        let f = Pv;
        let args = vec![
            Value::Number(Number::from_str("0.05").unwrap()),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(-1000)),
        ];
        let result = f.call(&args, &eval_ctx());
        let pv = result.as_number().unwrap();
        // Expected: approximately 7721.73
        let expected = Number::from_str("7721.73").unwrap();
        let diff = pv.sub(&expected).abs();
        assert!(diff < Number::from_str("1").unwrap());
    }

    #[test]
    fn test_fv_basic() {
        let f = Fv;
        let rate = Number::from_str("0.06").unwrap().checked_div(&Number::from_i64(12)).unwrap();
        let args = vec![
            Value::Number(rate),
            Value::Number(Number::from_i64(240)),
            Value::Number(Number::from_i64(-500)),
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(0)),
        ];
        let result = f.call(&args, &eval_ctx());
        let fv = result.as_number().unwrap();
        // Expected: approximately 231020.50
        assert!(fv > &Number::from_i64(230000));
        assert!(fv < &Number::from_i64(232000));
    }

    #[test]
    fn test_npv_basic() {
        let f = Npv;
        let args = vec![
            Value::Number(Number::from_str("0.10").unwrap()),
            Value::List(vec![
                Value::Number(Number::from_i64(30000)),
                Value::Number(Number::from_i64(40000)),
                Value::Number(Number::from_i64(50000)),
                Value::Number(Number::from_i64(30000)),
            ]),
        ];
        let result = f.call(&args, &eval_ctx());
        let npv = result.as_number().unwrap();
        // NPV of these flows at 10% should be around 117k
        assert!(npv > &Number::from_i64(100000));
    }

    #[test]
    fn test_irr_basic() {
        let f = Irr;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(-100000)),
            Value::Number(Number::from_i64(30000)),
            Value::Number(Number::from_i64(40000)),
            Value::Number(Number::from_i64(50000)),
            Value::Number(Number::from_i64(30000)),
        ])];
        let result = f.call(&args, &eval_ctx());
        let irr = result.as_number().unwrap();
        // IRR should be around 15-16%
        assert!(irr > &Number::from_str("0.10").unwrap());
        assert!(irr < &Number::from_str("0.20").unwrap());
    }
}
