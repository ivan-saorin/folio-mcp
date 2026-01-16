//! Investment return functions: cagr, roi, holding_period_return, annualized_return,
//! sharpe, sortino, max_drawdown, calmar, volatility, beta, alpha, treynor

use folio_plugin::prelude::*;
use crate::helpers::*;
use std::collections::HashMap;

// ============ CAGR ============

pub struct Cagr;

static CAGR_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "start_value",
        typ: "Number",
        description: "Starting value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end_value",
        typ: "Number",
        description: "Ending value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "years",
        typ: "Number",
        description: "Number of years",
        optional: false,
        default: None,
    },
];

static CAGR_EXAMPLES: [&str; 1] = ["cagr(10000, 25000, 5) → 0.2011"];

static CAGR_RELATED: [&str; 2] = ["annualized_return", "roi"];

impl FunctionPlugin for Cagr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cagr",
            description: "Compound annual growth rate: (end/start)^(1/years) - 1",
            usage: "cagr(start_value, end_value, years)",
            args: &CAGR_ARGS,
            returns: "Number",
            examples: &CAGR_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &CAGR_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("cagr", 3, args.len()));
        }

        let start = match extract_number(&args[0], "cagr", "start_value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let end = match extract_number(&args[1], "cagr", "end_value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let years = match extract_number(&args[2], "cagr", "years") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if start.is_zero() {
            return Value::Error(FolioError::domain_error("cagr: start_value must be non-zero"));
        }
        if years.is_zero() {
            return Value::Error(FolioError::domain_error("cagr: years must be non-zero"));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        let ratio = match end.checked_div(&start) {
            Ok(r) => r,
            Err(e) => return Value::Error(e.into()),
        };

        let exponent = match one.checked_div(&years) {
            Ok(e) => e,
            Err(e) => return Value::Error(e.into()),
        };

        let cagr = ratio.pow_real(&exponent, precision).sub(&one);
        Value::Number(cagr)
    }
}

// ============ ROI ============

pub struct Roi;

static ROI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "gain",
        typ: "Number",
        description: "Gain or profit",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "cost",
        typ: "Number",
        description: "Initial investment cost",
        optional: false,
        default: None,
    },
];

static ROI_EXAMPLES: [&str; 1] = ["roi(5000, 20000) → 0.25"];

static ROI_RELATED: [&str; 2] = ["cagr", "holding_period_return"];

impl FunctionPlugin for Roi {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "roi",
            description: "Simple return on investment: gain / cost",
            usage: "roi(gain, cost)",
            args: &ROI_ARGS,
            returns: "Number",
            examples: &ROI_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &ROI_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("roi", 2, args.len()));
        }

        let gain = match extract_number(&args[0], "roi", "gain") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let cost = match extract_number(&args[1], "roi", "cost") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if cost.is_zero() {
            return Value::Error(FolioError::domain_error("roi: cost must be non-zero"));
        }

        match gain.checked_div(&cost) {
            Ok(roi) => Value::Number(roi),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Holding Period Return ============

pub struct HoldingPeriodReturn;

static HPR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number>",
    description: "Series of values over time",
    optional: false,
    default: None,
}];

static HPR_EXAMPLES: [&str; 1] = ["holding_period_return([100, 105, 102, 110, 115]) → 0.15"];

static HPR_RELATED: [&str; 2] = ["cagr", "annualized_return"];

impl FunctionPlugin for HoldingPeriodReturn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "holding_period_return",
            description: "Total return: (final - initial) / initial",
            usage: "holding_period_return(values)",
            args: &HPR_ARGS,
            returns: "Number",
            examples: &HPR_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &HPR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("holding_period_return", 1, 0));
        }

        let values = match extract_numbers_from_list(&args[0], "holding_period_return", "values") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if values.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "holding_period_return: At least 2 values required",
            ));
        }

        let initial = &values[0];
        let final_val = &values[values.len() - 1];

        if initial.is_zero() {
            return Value::Error(FolioError::domain_error(
                "holding_period_return: initial value must be non-zero",
            ));
        }

        let hpr = final_val.sub(initial).checked_div(initial);
        match hpr {
            Ok(r) => Value::Number(r),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Annualized Return ============

pub struct AnnualizedReturn;

static ANN_RET_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "total_return",
        typ: "Number",
        description: "Total return over period",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "years",
        typ: "Number",
        description: "Number of years",
        optional: false,
        default: None,
    },
];

static ANN_RET_EXAMPLES: [&str; 1] = ["annualized_return(0.50, 3) → 0.1447"];

static ANN_RET_RELATED: [&str; 2] = ["cagr", "holding_period_return"];

impl FunctionPlugin for AnnualizedReturn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "annualized_return",
            description: "Annualize a total return: (1 + total_return)^(1/years) - 1",
            usage: "annualized_return(total_return, years)",
            args: &ANN_RET_ARGS,
            returns: "Number",
            examples: &ANN_RET_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &ANN_RET_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("annualized_return", 2, args.len()));
        }

        let total_return = match extract_number(&args[0], "annualized_return", "total_return") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let years = match extract_number(&args[1], "annualized_return", "years") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if years.is_zero() {
            return Value::Error(FolioError::domain_error("annualized_return: years must be non-zero"));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        let base = one.add(&total_return);
        let exponent = match one.checked_div(&years) {
            Ok(e) => e,
            Err(e) => return Value::Error(e.into()),
        };

        let result = base.pow_real(&exponent, precision).sub(&one);
        Value::Number(result)
    }
}

// ============ Sharpe Ratio ============

pub struct Sharpe;

static SHARPE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "returns",
        typ: "List<Number>",
        description: "Series of returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "risk_free_rate",
        typ: "Number",
        description: "Risk-free rate",
        optional: false,
        default: None,
    },
];

static SHARPE_EXAMPLES: [&str; 1] = ["sharpe([0.05, 0.02, -0.01, 0.08, 0.03], 0.02) → 0.567"];

static SHARPE_RELATED: [&str; 2] = ["sortino", "treynor"];

impl FunctionPlugin for Sharpe {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sharpe",
            description: "Sharpe ratio: (mean(returns) - risk_free) / stddev(returns)",
            usage: "sharpe(returns, risk_free_rate)",
            args: &SHARPE_ARGS,
            returns: "Number",
            examples: &SHARPE_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &SHARPE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("sharpe", 2, args.len()));
        }

        let returns = match extract_numbers_from_list(&args[0], "sharpe", "returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let risk_free = match extract_number(&args[1], "sharpe", "risk_free_rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if returns.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "sharpe: At least 2 returns required",
            ));
        }

        let precision = get_precision(ctx);
        let mean_return = mean(&returns);
        let std_dev = stddev(&returns, precision);

        if std_dev.is_zero() {
            return Value::Error(FolioError::domain_error(
                "sharpe: Standard deviation is zero",
            ));
        }

        let excess_return = mean_return.sub(&risk_free);
        match excess_return.checked_div(&std_dev) {
            Ok(ratio) => Value::Number(ratio),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Sortino Ratio ============

pub struct Sortino;

static SORTINO_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "returns",
        typ: "List<Number>",
        description: "Series of returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "risk_free_rate",
        typ: "Number",
        description: "Risk-free rate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "target",
        typ: "Number",
        description: "Target return (default: 0)",
        optional: true,
        default: Some("0"),
    },
];

static SORTINO_EXAMPLES: [&str; 1] = ["sortino([0.05, 0.02, -0.01, 0.08, 0.03], 0.02) → 0.823"];

static SORTINO_RELATED: [&str; 2] = ["sharpe", "max_drawdown"];

impl FunctionPlugin for Sortino {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sortino",
            description: "Sortino ratio (downside risk only)",
            usage: "sortino(returns, risk_free_rate, [target])",
            args: &SORTINO_ARGS,
            returns: "Number",
            examples: &SORTINO_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &SORTINO_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("sortino", 2, args.len()));
        }

        let returns = match extract_numbers_from_list(&args[0], "sortino", "returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let risk_free = match extract_number(&args[1], "sortino", "risk_free_rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let target = extract_optional_number(args, 2).unwrap_or_else(|| Number::from_i64(0));

        if returns.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "sortino: At least 2 returns required",
            ));
        }

        let precision = get_precision(ctx);
        let mean_return = mean(&returns);

        // Downside deviation: sqrt(mean of squared negative deviations from target)
        let downside_dev = downside_deviation(&returns, &target, precision);

        if downside_dev.is_zero() {
            return Value::Error(FolioError::domain_error(
                "sortino: Downside deviation is zero (no negative returns)",
            ));
        }

        let excess_return = mean_return.sub(&risk_free);
        match excess_return.checked_div(&downside_dev) {
            Ok(ratio) => Value::Number(ratio),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Max Drawdown ============

pub struct MaxDrawdown;

static MDD_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number>",
    description: "Series of values over time",
    optional: false,
    default: None,
}];

static MDD_EXAMPLES: [&str; 1] = ["max_drawdown([100, 110, 95, 105, 90, 115]) → {drawdown: -0.182, ...}"];

static MDD_RELATED: [&str; 2] = ["calmar", "volatility"];

impl FunctionPlugin for MaxDrawdown {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "max_drawdown",
            description: "Maximum peak-to-trough decline",
            usage: "max_drawdown(values)",
            args: &MDD_ARGS,
            returns: "Object",
            examples: &MDD_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &MDD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("max_drawdown", 1, 0));
        }

        let values = match extract_numbers_from_list(&args[0], "max_drawdown", "values") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if values.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "max_drawdown: At least 2 values required",
            ));
        }

        let (drawdown, peak_idx, trough_idx, recovery_idx) = calculate_max_drawdown(&values);

        let mut result = HashMap::new();
        result.insert("drawdown".to_string(), Value::Number(drawdown));
        result.insert("peak_index".to_string(), Value::Number(Number::from_i64(peak_idx as i64)));
        result.insert("trough_index".to_string(), Value::Number(Number::from_i64(trough_idx as i64)));
        result.insert("peak_value".to_string(), Value::Number(values[peak_idx].clone()));
        result.insert("trough_value".to_string(), Value::Number(values[trough_idx].clone()));
        if let Some(rec) = recovery_idx {
            result.insert("recovery_index".to_string(), Value::Number(Number::from_i64(rec as i64)));
        } else {
            result.insert("recovery_index".to_string(), Value::Null);
        }

        Value::Object(result)
    }
}

fn calculate_max_drawdown(values: &[Number]) -> (Number, usize, usize, Option<usize>) {
    let mut max_dd = Number::from_i64(0);
    let mut peak_idx = 0;
    let mut trough_idx = 0;
    let mut current_peak_idx = 0;
    let mut current_peak = values[0].clone();
    let mut recovery_idx = None;

    for (i, val) in values.iter().enumerate() {
        if val > &current_peak {
            current_peak = val.clone();
            current_peak_idx = i;
        }

        if !current_peak.is_zero() {
            let dd = val.sub(&current_peak).checked_div(&current_peak).unwrap_or_else(|_| Number::from_i64(0));
            if &dd < &max_dd {
                max_dd = dd;
                peak_idx = current_peak_idx;
                trough_idx = i;
                recovery_idx = None;
            }
        }
    }

    // Find recovery point (first value >= peak after trough)
    if trough_idx > 0 {
        let peak_value = &values[peak_idx];
        for (i, val) in values.iter().enumerate().skip(trough_idx + 1) {
            if val >= peak_value {
                recovery_idx = Some(i);
                break;
            }
        }
    }

    (max_dd, peak_idx, trough_idx, recovery_idx)
}

// ============ Calmar Ratio ============

pub struct Calmar;

static CALMAR_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Series of values over time",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "years",
        typ: "Number",
        description: "Number of years",
        optional: false,
        default: None,
    },
];

static CALMAR_EXAMPLES: [&str; 1] = ["calmar(values, 2) → 0.823"];

static CALMAR_RELATED: [&str; 2] = ["max_drawdown", "sharpe"];

impl FunctionPlugin for Calmar {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "calmar",
            description: "Calmar ratio: CAGR / |max drawdown|",
            usage: "calmar(values, years)",
            args: &CALMAR_ARGS,
            returns: "Number",
            examples: &CALMAR_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &CALMAR_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("calmar", 2, args.len()));
        }

        let values = match extract_numbers_from_list(&args[0], "calmar", "values") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let years = match extract_number(&args[1], "calmar", "years") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if values.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "calmar: At least 2 values required",
            ));
        }

        let precision = get_precision(ctx);
        let one = Number::from_i64(1);

        // Calculate CAGR
        let start = &values[0];
        let end = &values[values.len() - 1];

        if start.is_zero() {
            return Value::Error(FolioError::domain_error("calmar: start value must be non-zero"));
        }

        let ratio = match end.checked_div(start) {
            Ok(r) => r,
            Err(e) => return Value::Error(e.into()),
        };
        let exponent = match one.checked_div(&years) {
            Ok(e) => e,
            Err(e) => return Value::Error(e.into()),
        };
        let cagr = ratio.pow_real(&exponent, precision).sub(&one);

        // Calculate max drawdown
        let (max_dd, _, _, _) = calculate_max_drawdown(&values);
        let abs_dd = max_dd.abs();

        if abs_dd.is_zero() {
            return Value::Error(FolioError::domain_error(
                "calmar: Max drawdown is zero",
            ));
        }

        match cagr.checked_div(&abs_dd) {
            Ok(calmar) => Value::Number(calmar),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Volatility ============

pub struct Volatility;

static VOL_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "returns",
        typ: "List<Number>",
        description: "Series of returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "annualize",
        typ: "Bool",
        description: "Whether to annualize",
        optional: true,
        default: Some("true"),
    },
    ArgMeta {
        name: "periods_per_year",
        typ: "Number",
        description: "Number of periods per year",
        optional: true,
        default: Some("12"),
    },
];

static VOL_EXAMPLES: [&str; 1] = ["volatility(monthly_returns, true, 12) → 0.12"];

static VOL_RELATED: [&str; 2] = ["sharpe", "beta"];

impl FunctionPlugin for Volatility {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "volatility",
            description: "Annualized volatility (standard deviation of returns)",
            usage: "volatility(returns, [annualize], [periods_per_year])",
            args: &VOL_ARGS,
            returns: "Number",
            examples: &VOL_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &VOL_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("volatility", 1, 0));
        }

        let returns = match extract_numbers_from_list(&args[0], "volatility", "returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let annualize = match args.get(1) {
            Some(Value::Bool(b)) => *b,
            _ => true,
        };
        let periods_per_year = extract_int_or_default(args, 2, 12);

        if returns.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "volatility: At least 2 returns required",
            ));
        }

        let precision = get_precision(ctx);
        let std_dev = stddev(&returns, precision);

        if annualize {
            // Annualized vol = std_dev * sqrt(periods_per_year)
            match Number::from_i64(periods_per_year).sqrt(precision) {
                Ok(sqrt_periods) => Value::Number(std_dev.mul(&sqrt_periods)),
                Err(e) => Value::Error(e.into()),
            }
        } else {
            Value::Number(std_dev)
        }
    }
}

// ============ Beta ============

pub struct Beta;

static BETA_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "asset_returns",
        typ: "List<Number>",
        description: "Asset returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "market_returns",
        typ: "List<Number>",
        description: "Market returns",
        optional: false,
        default: None,
    },
];

static BETA_EXAMPLES: [&str; 1] = ["beta(stock_returns, sp500_returns) → 1.25"];

static BETA_RELATED: [&str; 2] = ["alpha", "treynor"];

impl FunctionPlugin for Beta {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "beta",
            description: "Beta coefficient: covariance(asset, market) / variance(market)",
            usage: "beta(asset_returns, market_returns)",
            args: &BETA_ARGS,
            returns: "Number",
            examples: &BETA_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &BETA_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("beta", 2, args.len()));
        }

        let asset = match extract_numbers_from_list(&args[0], "beta", "asset_returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let market = match extract_numbers_from_list(&args[1], "beta", "market_returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        if asset.len() != market.len() {
            return Value::Error(FolioError::domain_error(
                "beta: Asset and market returns must have same length",
            ));
        }

        if asset.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "beta: At least 2 returns required",
            ));
        }

        let precision = get_precision(ctx);
        let cov = covariance(&asset, &market);
        let var = variance(&market, precision);

        if var.is_zero() {
            return Value::Error(FolioError::domain_error(
                "beta: Market variance is zero",
            ));
        }

        match cov.checked_div(&var) {
            Ok(beta) => Value::Number(beta),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Alpha (Jensen's) ============

pub struct Alpha;

static ALPHA_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "asset_returns",
        typ: "List<Number>",
        description: "Asset returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "market_returns",
        typ: "List<Number>",
        description: "Market returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "risk_free_rate",
        typ: "Number",
        description: "Risk-free rate",
        optional: false,
        default: None,
    },
];

static ALPHA_EXAMPLES: [&str; 1] = ["alpha(stock_returns, sp500_returns, 0.02) → 0.015"];

static ALPHA_RELATED: [&str; 2] = ["beta", "treynor"];

impl FunctionPlugin for Alpha {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "alpha",
            description: "Jensen's alpha: mean(asset) - (risk_free + beta × (mean(market) - risk_free))",
            usage: "alpha(asset_returns, market_returns, risk_free_rate)",
            args: &ALPHA_ARGS,
            returns: "Number",
            examples: &ALPHA_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &ALPHA_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("alpha", 3, args.len()));
        }

        let asset = match extract_numbers_from_list(&args[0], "alpha", "asset_returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let market = match extract_numbers_from_list(&args[1], "alpha", "market_returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let risk_free = match extract_number(&args[2], "alpha", "risk_free_rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if asset.len() != market.len() {
            return Value::Error(FolioError::domain_error(
                "alpha: Asset and market returns must have same length",
            ));
        }

        let precision = get_precision(ctx);

        // Calculate beta
        let cov = covariance(&asset, &market);
        let var = variance(&market, precision);
        if var.is_zero() {
            return Value::Error(FolioError::domain_error(
                "alpha: Market variance is zero",
            ));
        }
        let beta = match cov.checked_div(&var) {
            Ok(b) => b,
            Err(e) => return Value::Error(e.into()),
        };

        // Calculate alpha
        let mean_asset = mean(&asset);
        let mean_market = mean(&market);
        let market_premium = mean_market.sub(&risk_free);
        let expected_return = risk_free.add(&beta.mul(&market_premium));
        let alpha = mean_asset.sub(&expected_return);

        Value::Number(alpha)
    }
}

// ============ Treynor Ratio ============

pub struct Treynor;

static TREYNOR_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "returns",
        typ: "List<Number>",
        description: "Asset returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "market_returns",
        typ: "List<Number>",
        description: "Market returns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "risk_free_rate",
        typ: "Number",
        description: "Risk-free rate",
        optional: false,
        default: None,
    },
];

static TREYNOR_EXAMPLES: [&str; 1] = ["treynor(stock_returns, sp500_returns, 0.02) → 0.08"];

static TREYNOR_RELATED: [&str; 2] = ["sharpe", "beta"];

impl FunctionPlugin for Treynor {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "treynor",
            description: "Treynor ratio: (mean(returns) - risk_free) / beta",
            usage: "treynor(returns, market_returns, risk_free_rate)",
            args: &TREYNOR_ARGS,
            returns: "Number",
            examples: &TREYNOR_EXAMPLES,
            category: "finance/returns",
            source: None,
            related: &TREYNOR_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("treynor", 3, args.len()));
        }

        let returns = match extract_numbers_from_list(&args[0], "treynor", "returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let market = match extract_numbers_from_list(&args[1], "treynor", "market_returns") {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };
        let risk_free = match extract_number(&args[2], "treynor", "risk_free_rate") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if returns.len() != market.len() {
            return Value::Error(FolioError::domain_error(
                "treynor: Asset and market returns must have same length",
            ));
        }

        let precision = get_precision(ctx);

        // Calculate beta
        let cov = covariance(&returns, &market);
        let var = variance(&market, precision);
        if var.is_zero() {
            return Value::Error(FolioError::domain_error(
                "treynor: Market variance is zero",
            ));
        }
        let beta = match cov.checked_div(&var) {
            Ok(b) => b,
            Err(e) => return Value::Error(e.into()),
        };

        if beta.is_zero() {
            return Value::Error(FolioError::domain_error(
                "treynor: Beta is zero",
            ));
        }

        let mean_return = mean(&returns);
        let excess_return = mean_return.sub(&risk_free);

        match excess_return.checked_div(&beta) {
            Ok(treynor) => Value::Number(treynor),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Helper functions ============

fn mean(numbers: &[Number]) -> Number {
    if numbers.is_empty() {
        return Number::from_i64(0);
    }
    let sum: Number = numbers.iter().fold(Number::from_i64(0), |acc, n| acc.add(n));
    sum.checked_div(&Number::from_i64(numbers.len() as i64)).unwrap_or_else(|_| Number::from_i64(0))
}

fn variance(numbers: &[Number], precision: u32) -> Number {
    if numbers.len() < 2 {
        return Number::from_i64(0);
    }

    let m = mean(numbers);
    let sum_sq: Number = numbers
        .iter()
        .map(|n| {
            let diff = n.sub(&m);
            diff.mul(&diff)
        })
        .fold(Number::from_i64(0), |acc, n| acc.add(&n));

    sum_sq.checked_div(&Number::from_i64((numbers.len() - 1) as i64)).unwrap_or_else(|_| Number::from_i64(0))
}

fn stddev(numbers: &[Number], precision: u32) -> Number {
    variance(numbers, precision).sqrt(precision).unwrap_or_else(|_| Number::from_i64(0))
}

fn covariance(x: &[Number], y: &[Number]) -> Number {
    if x.len() != y.len() || x.len() < 2 {
        return Number::from_i64(0);
    }

    let mean_x = mean(x);
    let mean_y = mean(y);

    let sum: Number = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| xi.sub(&mean_x).mul(&yi.sub(&mean_y)))
        .fold(Number::from_i64(0), |acc, n| acc.add(&n));

    sum.checked_div(&Number::from_i64((x.len() - 1) as i64)).unwrap_or_else(|_| Number::from_i64(0))
}

fn downside_deviation(returns: &[Number], target: &Number, precision: u32) -> Number {
    if returns.is_empty() {
        return Number::from_i64(0);
    }

    let sum_sq: Number = returns
        .iter()
        .filter_map(|r| {
            let diff = r.sub(target);
            if diff.is_negative() {
                Some(diff.mul(&diff))
            } else {
                None
            }
        })
        .fold(Number::from_i64(0), |acc, n| acc.add(&n));

    let count = returns.iter().filter(|r| r < &target).count();
    if count == 0 {
        return Number::from_i64(0);
    }

    sum_sq
        .checked_div(&Number::from_i64(count as i64))
        .ok()
        .and_then(|v| v.sqrt(precision).ok())
        .unwrap_or_else(|| Number::from_i64(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_cagr() {
        let f = Cagr;
        let args = vec![
            Value::Number(Number::from_i64(10000)),
            Value::Number(Number::from_i64(25000)),
            Value::Number(Number::from_i64(5)),
        ];
        let result = f.call(&args, &eval_ctx());
        let cagr = result.as_number().unwrap();
        // Expected: approximately 0.2011 (20.11%)
        let expected = Number::from_str("0.20").unwrap();
        let diff = cagr.sub(&expected).abs();
        assert!(diff < Number::from_str("0.01").unwrap());
    }

    #[test]
    fn test_roi() {
        let f = Roi;
        let args = vec![
            Value::Number(Number::from_i64(5000)),
            Value::Number(Number::from_i64(20000)),
        ];
        let result = f.call(&args, &eval_ctx());
        let roi = result.as_number().unwrap();
        assert_eq!(roi.as_decimal(2), "0.25");
    }

    #[test]
    fn test_holding_period_return() {
        let f = HoldingPeriodReturn;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(100)),
            Value::Number(Number::from_i64(115)),
        ])];
        let result = f.call(&args, &eval_ctx());
        let hpr = result.as_number().unwrap();
        assert_eq!(hpr.as_decimal(2), "0.15");
    }

    #[test]
    fn test_max_drawdown() {
        let values = vec![
            Number::from_i64(100),
            Number::from_i64(110),
            Number::from_i64(95),
            Number::from_i64(105),
            Number::from_i64(90),
            Number::from_i64(115),
        ];

        let (dd, peak_idx, trough_idx, _) = calculate_max_drawdown(&values);

        assert_eq!(peak_idx, 1); // Peak at 110
        assert_eq!(trough_idx, 4); // Trough at 90
        // Drawdown = (90 - 110) / 110 = -0.182
        let expected = Number::from_str("-0.18").unwrap();
        let diff = dd.sub(&expected).abs();
        assert!(diff < Number::from_str("0.01").unwrap());
    }
}
