//! Common financial utilities

use folio_core::{FolioError, Number, Value};

/// Extract a Number from a Value, returning error context
pub fn extract_number(value: &Value, func: &str, arg: &str) -> Result<Number, FolioError> {
    match value {
        Value::Number(n) => Ok(n.clone()),
        Value::Null => Err(FolioError::arg_type(func, arg, "Number", "Null")),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type(func, arg, "Number", other.type_name())),
    }
}

/// Extract optional Number (may be missing or null)
pub fn extract_optional_number(args: &[Value], index: usize) -> Option<Number> {
    args.get(index).and_then(|v| match v {
        Value::Number(n) => Some(n.clone()),
        _ => None,
    })
}

/// Extract optional integer parameter with default
pub fn extract_int_or_default(args: &[Value], index: usize, default: i64) -> i64 {
    args.get(index)
        .and_then(|v| match v {
            Value::Number(n) => n.to_i64(),
            _ => None,
        })
        .unwrap_or(default)
}

/// Extract numbers from a list Value
pub fn extract_numbers_from_list(value: &Value, func: &str, arg: &str) -> Result<Vec<Number>, FolioError> {
    match value {
        Value::List(items) => {
            let mut numbers = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Number(n) => numbers.push(n.clone()),
                    Value::Null => {} // Skip nulls in lists
                    Value::Error(e) => return Err(e.clone()),
                    other => {
                        return Err(FolioError::type_error(
                            "Number",
                            &format!("{}[{}]: {}", arg, i, other.type_name()),
                        ))
                    }
                }
            }
            Ok(numbers)
        }
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type(func, arg, "List<Number>", other.type_name())),
    }
}

/// Validate that a rate is > -1 (required for most financial calculations)
pub fn validate_rate(rate: &Number, func: &str) -> Result<(), FolioError> {
    let minus_one = Number::from_i64(-1);
    if rate <= &minus_one {
        return Err(FolioError::domain_error(format!(
            "{}(): rate must be greater than -1, got {}",
            func, rate
        )));
    }
    Ok(())
}

/// Validate that nper is positive
pub fn validate_nper(nper: &Number, func: &str) -> Result<(), FolioError> {
    if nper.is_zero() || nper.is_negative() {
        return Err(FolioError::domain_error(format!(
            "{}(): nper must be positive, got {}",
            func, nper
        )));
    }
    Ok(())
}

/// Newton-Raphson iteration for finding roots
/// Returns None if no convergence after max_iter
pub fn newton_raphson<F, D>(
    guess: Number,
    f: F,
    df: D,
    max_iter: usize,
    tol: &Number,
    precision: u32,
) -> Option<Number>
where
    F: Fn(&Number) -> Number,
    D: Fn(&Number) -> Number,
{
    let mut x = guess;

    for _ in 0..max_iter {
        let fx = f(&x);
        let dfx = df(&x);

        // Check for zero derivative
        if dfx.is_zero() {
            return None;
        }

        // x_new = x - f(x)/f'(x)
        let delta = fx.checked_div(&dfx).ok()?;
        let x_new = x.sub(&delta);

        // Check convergence
        let diff = x_new.sub(&x).abs();
        if &diff < tol {
            return Some(x_new);
        }

        x = x_new;
    }

    None
}

/// Calculate (1 + rate)^nper using arbitrary precision
pub fn compound_factor(rate: &Number, nper: &Number, precision: u32) -> Number {
    let one = Number::from_i64(1);
    let base = one.add(rate);

    // If nper is an integer, use exact power
    if let Some(n) = nper.to_i64() {
        if n.abs() <= i32::MAX as i64 {
            return base.pow(n as i32);
        }
    }

    // Otherwise use real power
    base.pow_real(nper, precision)
}

/// Get precision from context (default 50)
pub fn get_precision(ctx: &folio_plugin::EvalContext) -> u32 {
    ctx.precision
}

/// Days between two dates using ACT/365
pub fn days_between(start: &folio_core::FolioDateTime, end: &folio_core::FolioDateTime) -> i64 {
    let duration = end.duration_since(start);
    duration.as_days()
}

/// Year fraction using ACT/365
pub fn year_fraction_act365(start: &folio_core::FolioDateTime, end: &folio_core::FolioDateTime) -> Number {
    let days = days_between(start, end);
    Number::from_ratio(days, 365)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_number() {
        let val = Value::Number(Number::from_i64(42));
        let result = extract_number(&val, "test", "arg");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_i64(), Some(42));
    }

    #[test]
    fn test_extract_number_null_error() {
        let val = Value::Null;
        let result = extract_number(&val, "test", "arg");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rate() {
        let valid = Number::from_str("0.05").unwrap();
        assert!(validate_rate(&valid, "test").is_ok());

        let invalid = Number::from_i64(-2);
        assert!(validate_rate(&invalid, "test").is_err());
    }

    #[test]
    fn test_compound_factor() {
        // (1 + 0.1)^10 = 2.59374...
        let rate = Number::from_str("0.1").unwrap();
        let nper = Number::from_i64(10);
        let result = compound_factor(&rate, &nper, 50);

        // Should be approximately 2.59374
        let expected = Number::from_str("2.59374").unwrap();
        let diff = result.sub(&expected).abs();
        assert!(diff < Number::from_str("0.001").unwrap());
    }

    #[test]
    fn test_newton_raphson() {
        // Find sqrt(2) by solving x^2 - 2 = 0
        let f = |x: &Number| x.mul(x).sub(&Number::from_i64(2));
        let df = |x: &Number| x.mul(&Number::from_i64(2));

        let guess = Number::from_i64(1);
        let tol = Number::from_str("0.0000001").unwrap();

        let result = newton_raphson(guess, f, df, 100, &tol, 50);
        assert!(result.is_some());

        let sqrt2 = result.unwrap();
        let expected = Number::from_str("1.41421356").unwrap();
        let diff = sqrt2.sub(&expected).abs();
        assert!(diff < Number::from_str("0.0001").unwrap());
    }
}
