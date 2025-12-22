//! Folio Core - Fundamental types
//!
//! This crate provides the core types used throughout Folio:
//! - `Number`: Arbitrary precision rational numbers
//! - `Value`: Runtime values (numbers, text, objects, errors)
//! - `FolioError`: Structured errors for LLM consumption

mod number;
mod value;
mod error;

pub use number::{Number, NumberError};
pub use value::Value;
pub use error::{FolioError, ErrorContext, Severity, codes};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{Number, Value, FolioError, Severity};
    pub use crate::error::codes;
}

#[cfg(test)]
mod tests {
    use super::*;

    mod number_tests {
        use super::*;

        #[test]
        fn test_from_i64() {
            let n = Number::from_i64(42);
            assert_eq!(n.to_i64(), Some(42));
        }

        #[test]
        fn test_from_str_integer() {
            let n = Number::from_str("123").unwrap();
            assert_eq!(n.to_i64(), Some(123));
        }

        #[test]
        fn test_from_str_decimal() {
            let n = Number::from_str("3.14").unwrap();
            assert!(!n.is_integer());
        }

        #[test]
        fn test_from_str_fraction() {
            let n = Number::from_str("1/3").unwrap();
            assert!(!n.is_integer());
        }

        #[test]
        fn test_from_str_scientific() {
            let n = Number::from_str("1.5e2").unwrap();
            assert_eq!(n.to_i64(), Some(150));
        }

        #[test]
        fn test_from_str_scientific_integer_mantissa() {
            // Integer mantissa preserves full precision (no float64 intermediary)
            let avogadro = Number::from_str("602214076e15").unwrap();
            // Should be exactly 602214076 * 10^15
            let expected = Number::from_str("602214076000000000000000").unwrap();
            assert_eq!(avogadro.as_decimal(0), expected.as_decimal(0));

            // Negative exponent
            let h = Number::from_str("662607015e-42").unwrap();
            assert!(!h.is_zero());
            // Check it's a very small positive number
            let h_decimal = h.as_decimal(50);
            assert!(h_decimal.starts_with("0."), "Planck constant should be tiny: {}", h_decimal);
        }

        #[test]
        fn test_as_sigfigs() {
            // Large number - should use scientific notation
            let avogadro = Number::from_str("602214076e15").unwrap();
            let s = avogadro.as_sigfigs(4);
            assert!(s.contains("e23") || s.contains("e+23"), "Avogadro should be ~6e23: {}", s);
            assert!(s.starts_with("6.022"), "Should have 4 sig figs: {}", s);

            // Small number - should use scientific notation
            let h = Number::from_str("6.62607e-34").unwrap();
            let s = h.as_sigfigs(4);
            assert!(s.contains("e-3"), "Planck should use sci notation: {}", s);

            // Normal range number - regular notation
            let n = Number::from_str("123.456").unwrap();
            let s = n.as_sigfigs(4);
            assert_eq!(s, "123.5", "Normal number with 4 sigfigs: {}", s);

            // Small but in range
            let n = Number::from_str("0.001234").unwrap();
            let s = n.as_sigfigs(3);
            assert!(s.starts_with("0.00123"), "0.001234 with 3 sigfigs: {}", s);
        }

        #[test]
        fn test_ln_correctness() {
            // ln(100) should equal 2 * ln(10)
            let ten = Number::from_i64(10);
            let hundred = Number::from_i64(100);

            let ln_10 = ten.ln(50).unwrap();
            let ln_100 = hundred.ln(50).unwrap();
            let two_ln_10 = ln_10.mul(&Number::from_i64(2));

            // ln(10) ≈ 2.302585
            let ln_10_str = ln_10.as_decimal(5);
            assert!(ln_10_str.starts_with("2.3025"), "ln(10) should be ~2.3025, got: {}", ln_10_str);

            // ln(100) should equal 2*ln(10) ≈ 4.605170
            let ln_100_str = ln_100.as_decimal(5);
            let two_ln_10_str = two_ln_10.as_decimal(5);
            assert!(ln_100_str.starts_with("4.605"), "ln(100) should be ~4.605, got: {}", ln_100_str);
            assert_eq!(ln_100_str, two_ln_10_str, "ln(100) should equal 2*ln(10)");

            // Test larger number: ln(1000) = 3*ln(10) ≈ 6.907755
            let thousand = Number::from_i64(1000);
            let ln_1000 = thousand.ln(50).unwrap();
            let ln_1000_str = ln_1000.as_decimal(4);
            assert!(ln_1000_str.starts_with("6.907"), "ln(1000) should be ~6.907, got: {}", ln_1000_str);
        }

        #[test]
        fn test_exp_ln_identity() {
            // exp(ln(x)) should equal x
            let hundred = Number::from_i64(100);
            let ln_100 = hundred.ln(50).unwrap();
            let exp_ln_100 = ln_100.exp(50);
            let result_str = exp_ln_100.as_decimal(6);
            assert!(result_str.starts_with("100.000"),
                "exp(ln(100)) should be 100, got: {}", result_str);

            // Also test with a larger number
            let million = Number::from_i64(1000000);
            let ln_million = million.ln(50).unwrap();
            let exp_ln_million = ln_million.exp(50);
            let result_str = exp_ln_million.as_decimal(0);
            assert!(result_str.starts_with("1000000") || result_str.starts_with("999999"),
                "exp(ln(1000000)) should be ~1000000, got: {}", result_str);
        }

        #[test]
        fn test_pow_real_fractional() {
            // 4^0.5 = 2 (square root)
            let four = Number::from_i64(4);
            let half = Number::from_str("0.5").unwrap();
            let result = four.pow_real(&half, 50);
            let decimal = result.as_decimal(3);
            assert!(decimal.starts_with("2.0"), "4^0.5 should be 2, got: {}", decimal);

            // 8^(1/3) ≈ 2 (cube root)
            let eight = Number::from_i64(8);
            let third = Number::from_str("0.333333333333333").unwrap();
            let result = eight.pow_real(&third, 50);
            let decimal = result.as_decimal(2);
            assert!(decimal.starts_with("2.0") || decimal.starts_with("1.9"),
                "8^(1/3) should be ~2, got: {}", decimal);

            // 10^2.5 = 10^2 * 10^0.5 = 100 * 3.162... ≈ 316.2
            let ten = Number::from_i64(10);
            let two_point_five = Number::from_str("2.5").unwrap();
            let result = ten.pow_real(&two_point_five, 50);
            let decimal = result.as_decimal(1);
            assert!(decimal.starts_with("316."), "10^2.5 should be ~316.2, got: {}", decimal);
        }

        #[test]
        fn test_add() {
            let a = Number::from_i64(10);
            let b = Number::from_i64(32);
            assert_eq!(a.add(&b).to_i64(), Some(42));
        }

        #[test]
        fn test_sub() {
            let a = Number::from_i64(50);
            let b = Number::from_i64(8);
            assert_eq!(a.sub(&b).to_i64(), Some(42));
        }

        #[test]
        fn test_mul() {
            let a = Number::from_i64(6);
            let b = Number::from_i64(7);
            assert_eq!(a.mul(&b).to_i64(), Some(42));
        }

        #[test]
        fn test_checked_div() {
            let a = Number::from_i64(84);
            let b = Number::from_i64(2);
            assert_eq!(a.checked_div(&b).unwrap().to_i64(), Some(42));
        }

        #[test]
        fn test_div_by_zero() {
            let a = Number::from_i64(42);
            let b = Number::from_i64(0);
            assert!(a.checked_div(&b).is_err());
        }

        #[test]
        fn test_pow_positive() {
            let n = Number::from_i64(2);
            assert_eq!(n.pow(10).to_i64(), Some(1024));
        }

        #[test]
        fn test_pow_negative() {
            let n = Number::from_i64(2);
            let result = n.pow(-2);
            // 2^-2 = 1/4 = 0.25
            assert!(!result.is_integer());
        }

        #[test]
        fn test_pow_large_exponent() {
            // Test case: 1.003^300 ≈ 2.456 (compound interest factor)
            // This creates a large BigRational that overflows f64 individually
            // but the ratio should be representable
            let base = Number::from_str("1.003").unwrap();
            let result = base.pow(300);
            let decimal = result.as_decimal(2);
            // Should be approximately 2.46, not NaN or error
            assert!(decimal.starts_with("2.4"), "Expected ~2.4x, got: {}", decimal);
        }

        #[test]
        fn test_sqrt() {
            let n = Number::from_i64(4);
            let result = n.sqrt(50).unwrap();
            assert_eq!(result.to_i64(), Some(2));
        }

        #[test]
        fn test_sqrt_5() {
            // sqrt(5) ≈ 2.236
            let n = Number::from_i64(5);
            let result = n.sqrt(50).unwrap();
            assert!(!result.is_zero());
            let decimal = result.as_decimal(4);
            assert!(decimal.starts_with("2.236"), "sqrt(5) should be ~2.236, got: {}", decimal);
        }

        #[test]
        fn test_sqrt_negative() {
            let n = Number::from_i64(-4);
            assert!(n.sqrt(50).is_err());
        }

        #[test]
        fn test_phi() {
            let phi = Number::phi(50);
            // φ ≈ 1.618
            let decimal = phi.as_decimal(3);
            assert!(decimal.starts_with("1.618"));
        }

        #[test]
        fn test_pi() {
            let pi = Number::pi(50);
            let decimal = pi.as_decimal(5);
            // Pi = 3.14159..., rounded to 5 places = 3.14159
            assert!(decimal.starts_with("3.14159"), "Expected 3.14159..., got: {}", decimal);
        }

        #[test]
        fn test_e() {
            let e = Number::e(50);
            let decimal = e.as_decimal(3);
            assert!(decimal.starts_with("2.718"));
        }

        #[test]
        fn test_is_zero() {
            assert!(Number::from_i64(0).is_zero());
            assert!(!Number::from_i64(1).is_zero());
        }

        #[test]
        fn test_is_negative() {
            assert!(Number::from_i64(-5).is_negative());
            assert!(!Number::from_i64(5).is_negative());
            assert!(!Number::from_i64(0).is_negative());
        }

        #[test]
        fn test_abs() {
            assert_eq!(Number::from_i64(-42).abs().to_i64(), Some(42));
            assert_eq!(Number::from_i64(42).abs().to_i64(), Some(42));
        }

        #[test]
        fn test_small_number_display() {
            // Planck's constant: 6.62607015e-34
            let h = Number::from_str("6.62607015e-34").unwrap();
            let display = h.as_decimal(10);
            // Should show significant digits 6.63 at the end (rounded from 6.626...)
            // Format: 0.000...000663 with enough zeros to reach the significant digits
            assert!(display.ends_with("663") || display.ends_with("662"),
                "Very small number should show significant digits at end, got: {}", display);
            // Should have many leading zeros (34 decimal places for e-34)
            assert!(display.len() > 30, "Should have many decimal places, got: {}", display);
        }

        #[test]
        fn test_small_number_display_various() {
            // Test various small numbers - they should show 3 significant digits
            // Format is 0.000...XXX where XXX are the significant digits
            let n1 = Number::from_str("1e-10").unwrap();
            let d1 = n1.as_decimal(10);
            // 1e-10 = 0.0000000001 - should end with 1
            assert!(d1.ends_with("100"), "1e-10 should end with '100', got: {}", d1);

            let n2 = Number::from_str("2.5e-8").unwrap();
            let d2 = n2.as_decimal(10);
            // 2.5e-8 = 0.000000025 - should end with 250
            assert!(d2.ends_with("250") || d2.ends_with("25"), "2.5e-8 should end with '25x', got: {}", d2);

            let n3 = Number::from_str("1.23e-15").unwrap();
            let d3 = n3.as_decimal(10);
            // Should end with 123
            assert!(d3.ends_with("123"), "1.23e-15 should end with '123', got: {}", d3);
        }
    }

    mod value_tests {
        use super::*;

        #[test]
        fn test_from_i64() {
            let v: Value = 42i64.into();
            assert!(matches!(v, Value::Number(_)));
            assert_eq!(v.as_number().unwrap().to_i64(), Some(42));
        }

        #[test]
        fn test_from_str() {
            let v: Value = "hello".into();
            assert!(matches!(v, Value::Text(_)));
            assert_eq!(v.as_text(), Some("hello"));
        }

        #[test]
        fn test_from_bool() {
            let v: Value = true.into();
            assert!(matches!(v, Value::Bool(true)));
        }

        #[test]
        fn test_type_name() {
            assert_eq!(Value::Number(Number::from_i64(0)).type_name(), "Number");
            assert_eq!(Value::Text("".to_string()).type_name(), "Text");
            assert_eq!(Value::Bool(true).type_name(), "Bool");
            assert_eq!(Value::Null.type_name(), "Null");
        }

        #[test]
        fn test_is_error() {
            let err = Value::Error(FolioError::div_zero());
            assert!(err.is_error());
            assert!(!Value::Null.is_error());
        }

        #[test]
        fn test_to_number_from_text() {
            let v = Value::Text("42".to_string());
            let n = v.to_number();
            assert!(matches!(n, Value::Number(_)));
        }

        #[test]
        fn test_to_bool_truthy() {
            assert!(matches!(Value::Number(Number::from_i64(1)).to_bool(), Value::Bool(true)));
            assert!(matches!(Value::Number(Number::from_i64(0)).to_bool(), Value::Bool(false)));
            assert!(matches!(Value::Text("hi".to_string()).to_bool(), Value::Bool(true)));
            assert!(matches!(Value::Text("".to_string()).to_bool(), Value::Bool(false)));
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_error_construction() {
            let err = FolioError::div_zero();
            assert_eq!(err.code, codes::DIV_ZERO);
        }

        #[test]
        fn test_error_with_context() {
            let err = FolioError::undefined_var("x")
                .in_cell("result")
                .with_formula("x + 1");
            assert!(err.context.is_some());
            let ctx = err.context.unwrap();
            assert_eq!(ctx.cell, Some("result".to_string()));
            assert_eq!(ctx.formula, Some("x + 1".to_string()));
        }

        #[test]
        fn test_error_with_note() {
            let err = FolioError::type_error("Number", "Text")
                .with_note("from left operand");
            let ctx = err.context.unwrap();
            assert_eq!(ctx.notes.len(), 1);
            assert_eq!(ctx.notes[0], "from left operand");
        }

        #[test]
        fn test_error_display() {
            let err = FolioError::parse_error("unexpected token");
            let display = format!("{}", err);
            assert!(display.contains("PARSE_ERROR"));
        }
    }
}
