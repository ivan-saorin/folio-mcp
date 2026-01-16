//! Common kitchen utilities

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

/// Extract a Text string from a Value
pub fn extract_text(value: &Value, func: &str, arg: &str) -> Result<String, FolioError> {
    match value {
        Value::Text(s) => Ok(s.clone()),
        Value::Null => Err(FolioError::arg_type(func, arg, "Text", "Null")),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type(func, arg, "Text", other.type_name())),
    }
}

/// Extract optional Number (may be missing or null)
pub fn extract_optional_number(args: &[Value], index: usize) -> Option<Number> {
    args.get(index).and_then(|v| match v {
        Value::Number(n) => Some(n.clone()),
        _ => None,
    })
}

/// Extract optional Text string
pub fn extract_optional_text(args: &[Value], index: usize) -> Option<String> {
    args.get(index).and_then(|v| match v {
        Value::Text(s) => Some(s.clone()),
        _ => None,
    })
}

/// Normalize ingredient name (lowercase, trim, standardize)
pub fn normalize_ingredient(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace('-', " ")
        .replace('_', " ")
}

/// Validate positive number
pub fn validate_positive(value: &Number, func: &str, arg: &str) -> Result<(), FolioError> {
    if value.is_negative() || value.is_zero() {
        return Err(FolioError::domain_error(format!(
            "{}(): {} must be positive, got {}",
            func, arg, value
        )));
    }
    Ok(())
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
    fn test_extract_text() {
        let val = Value::Text("hello".to_string());
        let result = extract_text(&val, "test", "arg");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_normalize_ingredient() {
        assert_eq!(normalize_ingredient("All-Purpose Flour"), "all purpose flour");
        assert_eq!(normalize_ingredient("  brown_sugar  "), "brown sugar");
        assert_eq!(normalize_ingredient("BUTTER"), "butter");
    }
}
