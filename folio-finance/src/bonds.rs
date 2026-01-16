//! Bond calculation functions: bond_price, bond_yield, duration, mduration, convexity, accrint

use folio_plugin::prelude::*;
use crate::helpers::*;

// ============ BondPrice ============

pub struct BondPrice;

static BOND_PRICE_ARGS: [ArgMeta; 7] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual coupon rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "yld",
        typ: "Number",
        description: "Annual yield to maturity",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "redemption",
        typ: "Number",
        description: "Redemption value per 100 face",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "frequency",
        typ: "Number",
        description: "Coupon payments per year (1, 2, 4)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "settlement",
        typ: "DateTime",
        description: "Settlement date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "maturity",
        typ: "DateTime",
        description: "Maturity date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "day_count",
        typ: "Text",
        description: "Day count convention",
        optional: true,
        default: Some("30/360"),
    },
];

static BOND_PRICE_EXAMPLES: [&str; 1] = [
    "bond_price(0.05, 0.06, 100, 2, settlement, maturity) → 92.56",
];

static BOND_PRICE_RELATED: [&str; 3] = ["bond_yield", "duration", "accrint"];

impl FunctionPlugin for BondPrice {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "bond_price",
            description: "Bond price per 100 face value",
            usage: "bond_price(rate, yld, redemption, frequency, settlement, maturity, [day_count])",
            args: &BOND_PRICE_ARGS,
            returns: "Number",
            examples: &BOND_PRICE_EXAMPLES,
            category: "finance/bonds",
            source: None,
            related: &BOND_PRICE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 6 {
            return Value::Error(FolioError::arg_count("bond_price", 6, args.len()));
        }

        let rate = match extract_number(&args[0], "bond_price", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let yld = match extract_number(&args[1], "bond_price", "yld") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let redemption = match extract_number(&args[2], "bond_price", "redemption") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let frequency = match extract_number(&args[3], "bond_price", "frequency") {
            Ok(n) => n.to_i64().unwrap_or(2),
            Err(e) => return Value::Error(e),
        };
        let settlement = match &args[4] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "bond_price",
                "settlement",
                "DateTime",
                other.type_name(),
            )),
        };
        let maturity = match &args[5] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "bond_price",
                "maturity",
                "DateTime",
                other.type_name(),
            )),
        };

        let precision = get_precision(ctx);
        match calculate_bond_price(&rate, &yld, &redemption, frequency, &settlement, &maturity, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_bond_price(
    rate: &Number,
    yld: &Number,
    redemption: &Number,
    frequency: i64,
    settlement: &FolioDateTime,
    maturity: &FolioDateTime,
    precision: u32,
) -> Result<Number, FolioError> {
    let freq_num = Number::from_i64(frequency);
    let one = Number::from_i64(1);
    let hundred = Number::from_i64(100);

    // Coupon per period
    let coupon = rate.mul(&hundred).checked_div(&freq_num)?;

    // Yield per period
    let yld_per_period = yld.checked_div(&freq_num)?;

    // Number of periods remaining (approximate)
    let days_to_maturity = days_between(settlement, maturity);
    let days_per_period = 365 / frequency;
    let n_periods = (days_to_maturity as f64 / days_per_period as f64).ceil() as i64;

    if n_periods <= 0 {
        return Ok(redemption.clone());
    }

    // Fractional period from settlement to next coupon
    let fraction = Number::from_str("1.0").unwrap(); // Simplified - assume full period

    // Price = sum of PV of coupons + PV of redemption
    let mut price = Number::from_i64(0);
    let discount_factor = one.add(&yld_per_period);

    // PV of coupon payments
    for i in 1..=n_periods {
        let periods = Number::from_i64(i);
        let discount = compound_factor(&yld_per_period, &periods, precision);
        let pv = coupon.checked_div(&discount)?;
        price = price.add(&pv);
    }

    // PV of redemption
    let n_periods_num = Number::from_i64(n_periods);
    let redemption_discount = compound_factor(&yld_per_period, &n_periods_num, precision);
    let pv_redemption = redemption.checked_div(&redemption_discount)?;
    price = price.add(&pv_redemption);

    Ok(price)
}

// ============ BondYield ============

pub struct BondYield;

static BOND_YIELD_ARGS: [ArgMeta; 8] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual coupon rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "price",
        typ: "Number",
        description: "Bond price per 100 face",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "redemption",
        typ: "Number",
        description: "Redemption value per 100 face",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "frequency",
        typ: "Number",
        description: "Coupon payments per year",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "settlement",
        typ: "DateTime",
        description: "Settlement date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "maturity",
        typ: "DateTime",
        description: "Maturity date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "day_count",
        typ: "Text",
        description: "Day count convention",
        optional: true,
        default: Some("30/360"),
    },
    ArgMeta {
        name: "guess",
        typ: "Number",
        description: "Initial guess",
        optional: true,
        default: Some("0.05"),
    },
];

static BOND_YIELD_EXAMPLES: [&str; 1] = [
    "bond_yield(0.05, 92.56, 100, 2, settlement, maturity) → 0.06",
];

static BOND_YIELD_RELATED: [&str; 2] = ["bond_price", "duration"];

impl FunctionPlugin for BondYield {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "bond_yield",
            description: "Yield to maturity using Newton-Raphson",
            usage: "bond_yield(rate, price, redemption, frequency, settlement, maturity, [day_count], [guess])",
            args: &BOND_YIELD_ARGS,
            returns: "Number",
            examples: &BOND_YIELD_EXAMPLES,
            category: "finance/bonds",
            source: None,
            related: &BOND_YIELD_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 6 {
            return Value::Error(FolioError::arg_count("bond_yield", 6, args.len()));
        }

        let rate = match extract_number(&args[0], "bond_yield", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let price = match extract_number(&args[1], "bond_yield", "price") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let redemption = match extract_number(&args[2], "bond_yield", "redemption") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let frequency = match extract_number(&args[3], "bond_yield", "frequency") {
            Ok(n) => n.to_i64().unwrap_or(2),
            Err(e) => return Value::Error(e),
        };
        let settlement = match &args[4] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "bond_yield",
                "settlement",
                "DateTime",
                other.type_name(),
            )),
        };
        let maturity = match &args[5] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "bond_yield",
                "maturity",
                "DateTime",
                other.type_name(),
            )),
        };
        let guess = extract_optional_number(args, 7)
            .unwrap_or_else(|| Number::from_str("0.05").unwrap());

        let precision = get_precision(ctx);

        // Use Newton-Raphson to find yield
        let f = |yld: &Number| -> Number {
            calculate_bond_price(&rate, yld, &redemption, frequency, &settlement, &maturity, precision)
                .map(|p| p.sub(&price))
                .unwrap_or_else(|_| Number::from_i64(0))
        };

        let df = |yld: &Number| -> Number {
            let eps = Number::from_str("0.0001").unwrap();
            let f_plus = f(&yld.add(&eps));
            let f_minus = f(&yld.sub(&eps));
            let two_eps = eps.mul(&Number::from_i64(2));
            f_plus.sub(&f_minus).checked_div(&two_eps).unwrap_or_else(|_| Number::from_i64(1))
        };

        let tol = Number::from_str("0.000000001").unwrap();
        match newton_raphson(guess, f, df, 100, &tol, precision) {
            Some(yld) => Value::Number(yld),
            None => Value::Error(FolioError::domain_error(
                "bond_yield: Failed to converge. Try a different initial guess.",
            )),
        }
    }
}

// ============ Duration (Macaulay) ============

pub struct Duration;

static DURATION_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual coupon rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "yld",
        typ: "Number",
        description: "Annual yield to maturity",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "frequency",
        typ: "Number",
        description: "Coupon payments per year",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "settlement",
        typ: "DateTime",
        description: "Settlement date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "maturity",
        typ: "DateTime",
        description: "Maturity date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "day_count",
        typ: "Text",
        description: "Day count convention",
        optional: true,
        default: Some("30/360"),
    },
];

static DURATION_EXAMPLES: [&str; 1] = [
    "duration(0.05, 0.06, 2, settlement, maturity) → 7.89",
];

static DURATION_RELATED: [&str; 3] = ["mduration", "convexity", "bond_price"];

impl FunctionPlugin for Duration {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "duration",
            description: "Macaulay duration in years",
            usage: "duration(rate, yld, frequency, settlement, maturity, [day_count])",
            args: &DURATION_ARGS,
            returns: "Number",
            examples: &DURATION_EXAMPLES,
            category: "finance/bonds",
            source: None,
            related: &DURATION_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 5 {
            return Value::Error(FolioError::arg_count("duration", 5, args.len()));
        }

        let rate = match extract_number(&args[0], "duration", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let yld = match extract_number(&args[1], "duration", "yld") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let frequency = match extract_number(&args[2], "duration", "frequency") {
            Ok(n) => n.to_i64().unwrap_or(2),
            Err(e) => return Value::Error(e),
        };
        let settlement = match &args[3] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "duration",
                "settlement",
                "DateTime",
                other.type_name(),
            )),
        };
        let maturity = match &args[4] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "duration",
                "maturity",
                "DateTime",
                other.type_name(),
            )),
        };

        let precision = get_precision(ctx);
        let redemption = Number::from_i64(100);

        match calculate_duration(&rate, &yld, &redemption, frequency, &settlement, &maturity, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_duration(
    rate: &Number,
    yld: &Number,
    redemption: &Number,
    frequency: i64,
    settlement: &FolioDateTime,
    maturity: &FolioDateTime,
    precision: u32,
) -> Result<Number, FolioError> {
    let freq_num = Number::from_i64(frequency);
    let one = Number::from_i64(1);
    let hundred = Number::from_i64(100);

    // Coupon per period
    let coupon = rate.mul(&hundred).checked_div(&freq_num)?;

    // Yield per period
    let yld_per_period = yld.checked_div(&freq_num)?;

    // Number of periods
    let days_to_maturity = days_between(settlement, maturity);
    let days_per_period = 365 / frequency;
    let n_periods = (days_to_maturity as f64 / days_per_period as f64).ceil() as i64;

    if n_periods <= 0 {
        return Ok(Number::from_i64(0));
    }

    // Weighted sum of cash flows
    let mut weighted_sum = Number::from_i64(0);
    let mut price = Number::from_i64(0);

    for i in 1..=n_periods {
        let periods = Number::from_i64(i);
        let time_in_years = periods.checked_div(&freq_num)?;
        let discount = compound_factor(&yld_per_period, &periods, precision);
        let pv = coupon.checked_div(&discount)?;
        price = price.add(&pv);
        weighted_sum = weighted_sum.add(&pv.mul(&time_in_years));
    }

    // Add redemption
    let n_periods_num = Number::from_i64(n_periods);
    let time_to_maturity = n_periods_num.checked_div(&freq_num)?;
    let redemption_discount = compound_factor(&yld_per_period, &n_periods_num, precision);
    let pv_redemption = redemption.checked_div(&redemption_discount)?;
    price = price.add(&pv_redemption);
    weighted_sum = weighted_sum.add(&pv_redemption.mul(&time_to_maturity));

    // Duration = weighted sum / price
    Ok(weighted_sum.checked_div(&price)?)
}

// ============ Modified Duration ============

pub struct Mduration;

static MDURATION_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual coupon rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "yld",
        typ: "Number",
        description: "Annual yield to maturity",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "frequency",
        typ: "Number",
        description: "Coupon payments per year",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "settlement",
        typ: "DateTime",
        description: "Settlement date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "maturity",
        typ: "DateTime",
        description: "Maturity date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "day_count",
        typ: "Text",
        description: "Day count convention",
        optional: true,
        default: Some("30/360"),
    },
];

static MDURATION_EXAMPLES: [&str; 1] = [
    "mduration(0.05, 0.06, 2, settlement, maturity) → 7.66",
];

static MDURATION_RELATED: [&str; 2] = ["duration", "convexity"];

impl FunctionPlugin for Mduration {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mduration",
            description: "Modified duration: Macaulay duration / (1 + yld/frequency)",
            usage: "mduration(rate, yld, frequency, settlement, maturity, [day_count])",
            args: &MDURATION_ARGS,
            returns: "Number",
            examples: &MDURATION_EXAMPLES,
            category: "finance/bonds",
            source: None,
            related: &MDURATION_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 5 {
            return Value::Error(FolioError::arg_count("mduration", 5, args.len()));
        }

        let rate = match extract_number(&args[0], "mduration", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let yld = match extract_number(&args[1], "mduration", "yld") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let frequency = match extract_number(&args[2], "mduration", "frequency") {
            Ok(n) => n.to_i64().unwrap_or(2),
            Err(e) => return Value::Error(e),
        };
        let settlement = match &args[3] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "mduration",
                "settlement",
                "DateTime",
                other.type_name(),
            )),
        };
        let maturity = match &args[4] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "mduration",
                "maturity",
                "DateTime",
                other.type_name(),
            )),
        };

        let precision = get_precision(ctx);
        let redemption = Number::from_i64(100);

        let duration = match calculate_duration(&rate, &yld, &redemption, frequency, &settlement, &maturity, precision) {
            Ok(d) => d,
            Err(e) => return Value::Error(e),
        };

        // Modified duration = Macaulay duration / (1 + yld/frequency)
        let freq_num = Number::from_i64(frequency);
        let one = Number::from_i64(1);
        let divisor = one.add(&yld.checked_div(&freq_num).unwrap_or_else(|_| Number::from_i64(0)));

        match duration.checked_div(&divisor) {
            Ok(md) => Value::Number(md),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Convexity ============

pub struct Convexity;

static CONVEXITY_ARGS: [ArgMeta; 6] = [
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual coupon rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "yld",
        typ: "Number",
        description: "Annual yield to maturity",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "frequency",
        typ: "Number",
        description: "Coupon payments per year",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "settlement",
        typ: "DateTime",
        description: "Settlement date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "maturity",
        typ: "DateTime",
        description: "Maturity date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "day_count",
        typ: "Text",
        description: "Day count convention",
        optional: true,
        default: Some("30/360"),
    },
];

static CONVEXITY_EXAMPLES: [&str; 1] = [
    "convexity(0.05, 0.06, 2, settlement, maturity) → 72.34",
];

static CONVEXITY_RELATED: [&str; 2] = ["duration", "mduration"];

impl FunctionPlugin for Convexity {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "convexity",
            description: "Bond convexity",
            usage: "convexity(rate, yld, frequency, settlement, maturity, [day_count])",
            args: &CONVEXITY_ARGS,
            returns: "Number",
            examples: &CONVEXITY_EXAMPLES,
            category: "finance/bonds",
            source: None,
            related: &CONVEXITY_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 5 {
            return Value::Error(FolioError::arg_count("convexity", 5, args.len()));
        }

        let rate = match extract_number(&args[0], "convexity", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let yld = match extract_number(&args[1], "convexity", "yld") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let frequency = match extract_number(&args[2], "convexity", "frequency") {
            Ok(n) => n.to_i64().unwrap_or(2),
            Err(e) => return Value::Error(e),
        };
        let settlement = match &args[3] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "convexity",
                "settlement",
                "DateTime",
                other.type_name(),
            )),
        };
        let maturity = match &args[4] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "convexity",
                "maturity",
                "DateTime",
                other.type_name(),
            )),
        };

        let precision = get_precision(ctx);
        let redemption = Number::from_i64(100);

        match calculate_convexity(&rate, &yld, &redemption, frequency, &settlement, &maturity, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

fn calculate_convexity(
    rate: &Number,
    yld: &Number,
    redemption: &Number,
    frequency: i64,
    settlement: &FolioDateTime,
    maturity: &FolioDateTime,
    precision: u32,
) -> Result<Number, FolioError> {
    let freq_num = Number::from_i64(frequency);
    let one = Number::from_i64(1);
    let two = Number::from_i64(2);
    let hundred = Number::from_i64(100);

    // Coupon per period
    let coupon = rate.mul(&hundred).checked_div(&freq_num)?;

    // Yield per period
    let yld_per_period = yld.checked_div(&freq_num)?;

    // Number of periods
    let days_to_maturity = days_between(settlement, maturity);
    let days_per_period = 365 / frequency;
    let n_periods = (days_to_maturity as f64 / days_per_period as f64).ceil() as i64;

    if n_periods <= 0 {
        return Ok(Number::from_i64(0));
    }

    // Calculate convexity: sum of t*(t+1)*PV(CF) / (P * (1+y)^2)
    let mut weighted_sum = Number::from_i64(0);
    let mut price = Number::from_i64(0);

    for i in 1..=n_periods {
        let t = Number::from_i64(i);
        let t_plus_1 = t.add(&one);
        let discount = compound_factor(&yld_per_period, &t, precision);
        let pv = coupon.checked_div(&discount)?;
        price = price.add(&pv);
        weighted_sum = weighted_sum.add(&pv.mul(&t).mul(&t_plus_1));
    }

    // Add redemption
    let n = Number::from_i64(n_periods);
    let n_plus_1 = n.add(&one);
    let redemption_discount = compound_factor(&yld_per_period, &n, precision);
    let pv_redemption = redemption.checked_div(&redemption_discount)?;
    price = price.add(&pv_redemption);
    weighted_sum = weighted_sum.add(&pv_redemption.mul(&n).mul(&n_plus_1));

    // Convexity = weighted_sum / (price * (1+y)^2 * freq^2)
    let yld_factor = one.add(&yld_per_period).pow(2);
    let freq_squared = freq_num.pow(2);
    let denominator = price.mul(&yld_factor).mul(&freq_squared);

    Ok(weighted_sum.checked_div(&denominator)?)
}

// ============ Accrued Interest ============

pub struct Accrint;

static ACCRINT_ARGS: [ArgMeta; 7] = [
    ArgMeta {
        name: "issue",
        typ: "DateTime",
        description: "Issue date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "first_interest",
        typ: "DateTime",
        description: "First interest payment date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "settlement",
        typ: "DateTime",
        description: "Settlement date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "rate",
        typ: "Number",
        description: "Annual coupon rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "par",
        typ: "Number",
        description: "Par value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "frequency",
        typ: "Number",
        description: "Coupon payments per year",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "day_count",
        typ: "Text",
        description: "Day count convention",
        optional: true,
        default: Some("30/360"),
    },
];

static ACCRINT_EXAMPLES: [&str; 1] = [
    "accrint(issue, first_interest, settlement, 0.05, 1000, 2) → 23.61",
];

static ACCRINT_RELATED: [&str; 2] = ["bond_price", "bond_yield"];

impl FunctionPlugin for Accrint {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "accrint",
            description: "Accrued interest for a bond",
            usage: "accrint(issue, first_interest, settlement, rate, par, frequency, [day_count])",
            args: &ACCRINT_ARGS,
            returns: "Number",
            examples: &ACCRINT_EXAMPLES,
            category: "finance/bonds",
            source: None,
            related: &ACCRINT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 6 {
            return Value::Error(FolioError::arg_count("accrint", 6, args.len()));
        }

        let issue = match &args[0] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "accrint",
                "issue",
                "DateTime",
                other.type_name(),
            )),
        };
        let first_interest = match &args[1] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "accrint",
                "first_interest",
                "DateTime",
                other.type_name(),
            )),
        };
        let settlement = match &args[2] {
            Value::DateTime(dt) => dt.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "accrint",
                "settlement",
                "DateTime",
                other.type_name(),
            )),
        };
        let rate = match extract_number(&args[3], "accrint", "rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let par = match extract_number(&args[4], "accrint", "par") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let frequency = match extract_number(&args[5], "accrint", "frequency") {
            Ok(n) => n.to_i64().unwrap_or(2),
            Err(e) => return Value::Error(e),
        };

        // Days from issue (or last coupon) to settlement
        let days_accrued = days_between(&issue, &settlement);
        let days_in_period = 365 / frequency;

        // Accrued interest = par * rate * (days_accrued / days_in_year)
        let year_fraction = Number::from_ratio(days_accrued, 365);
        let accrued = par.mul(&rate).mul(&year_fraction);

        Value::Number(accrued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_bond_price_par() {
        // When yield = coupon rate, price should be close to 100
        let rate = Number::from_str("0.05").unwrap();
        let yld = Number::from_str("0.05").unwrap();
        let redemption = Number::from_i64(100);

        let settlement = FolioDateTime::from_ymd(2024, 1, 15).unwrap();
        let maturity = FolioDateTime::from_ymd(2034, 1, 15).unwrap();

        let price = calculate_bond_price(&rate, &yld, &redemption, 2, &settlement, &maturity, 50).unwrap();

        // Should be approximately 100
        let diff = price.sub(&Number::from_i64(100)).abs();
        assert!(diff < Number::from_i64(5)); // Allow some rounding
    }

    #[test]
    fn test_duration_positive() {
        let rate = Number::from_str("0.05").unwrap();
        let yld = Number::from_str("0.06").unwrap();
        let redemption = Number::from_i64(100);

        let settlement = FolioDateTime::from_ymd(2024, 1, 15).unwrap();
        let maturity = FolioDateTime::from_ymd(2034, 1, 15).unwrap();

        let duration = calculate_duration(&rate, &yld, &redemption, 2, &settlement, &maturity, 50).unwrap();

        // Duration should be positive and less than maturity
        assert!(!duration.is_negative());
        assert!(duration < Number::from_i64(10));
    }
}
