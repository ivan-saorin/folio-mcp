//! Helper functions for text operations
//!
//! Common utilities for extracting and validating text inputs.

use folio_core::{Value, FolioError};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Extract text from a Value, handling null propagation
pub fn extract_text(value: &Value) -> Result<Option<&str>, FolioError> {
    match value {
        Value::Text(s) => Ok(Some(s.as_str())),
        Value::Null => Ok(None),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::type_error("Text", other.type_name())),
    }
}

/// Extract text, returning error if null
pub fn require_text<'a>(value: &'a Value, func: &str, arg: &str) -> Result<&'a str, FolioError> {
    match value {
        Value::Text(s) => Ok(s.as_str()),
        Value::Null => Err(FolioError::arg_type(func, arg, "Text", "Null")),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type(func, arg, "Text", other.type_name())),
    }
}

/// Extract optional text (may be missing or null)
pub fn extract_optional_text<'a>(args: &'a [Value], index: usize) -> Option<&'a str> {
    args.get(index).and_then(|v| match v {
        Value::Text(s) => Some(s.as_str()),
        _ => None,
    })
}

/// Extract integer from Value
pub fn extract_int(value: &Value, func: &str, arg: &str) -> Result<i64, FolioError> {
    match value {
        Value::Number(n) => n.to_i64().ok_or_else(|| {
            FolioError::domain_error(format!("{}(): {} must be a valid integer", func, arg))
        }),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type(func, arg, "Number", other.type_name())),
    }
}

/// Extract optional integer with default
pub fn extract_int_or(args: &[Value], index: usize, default: i64, func: &str, arg: &str) -> Result<i64, FolioError> {
    match args.get(index) {
        Some(Value::Number(n)) => n.to_i64().ok_or_else(|| {
            FolioError::domain_error(format!("{}(): {} must be a valid integer", func, arg))
        }),
        Some(Value::Null) | None => Ok(default),
        Some(Value::Error(e)) => Err(e.clone()),
        Some(other) => Err(FolioError::arg_type(func, arg, "Number", other.type_name())),
    }
}

/// Regex cache for compiled patterns
static REGEX_CACHE: OnceLock<RwLock<HashMap<String, Regex>>> = OnceLock::new();

fn get_cache() -> &'static RwLock<HashMap<String, Regex>> {
    REGEX_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Get or compile a regex pattern with caching
pub fn get_regex(pattern: &str) -> Result<Regex, FolioError> {
    let cache = get_cache();

    // Try read lock first
    {
        let read_guard = cache.read().map_err(|_| {
            FolioError::new("INTERNAL", "Failed to acquire regex cache lock")
        })?;
        if let Some(re) = read_guard.get(pattern) {
            return Ok(re.clone());
        }
    }

    // Compile the regex
    let re = Regex::new(pattern).map_err(|e| {
        FolioError::parse_error(format!("Invalid regex '{}': {}", pattern, e))
    })?;

    // Try to cache it (don't fail if we can't)
    if let Ok(mut write_guard) = cache.write() {
        write_guard.insert(pattern.to_string(), re.clone());
    }

    Ok(re)
}

/// Convert character index to byte position (handling Unicode)
pub fn char_to_byte_index(s: &str, char_idx: usize) -> Option<usize> {
    s.char_indices().nth(char_idx).map(|(i, _)| i)
}

/// Normalize index to handle negative values (Python-style)
pub fn normalize_index(idx: i64, len: usize) -> usize {
    if idx < 0 {
        let positive = (-idx) as usize;
        if positive > len {
            0
        } else {
            len - positive
        }
    } else {
        (idx as usize).min(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use folio_core::Number;

    #[test]
    fn test_extract_text() {
        let val = Value::Text("hello".to_string());
        assert_eq!(extract_text(&val).unwrap(), Some("hello"));
    }

    #[test]
    fn test_extract_text_null() {
        let val = Value::Null;
        assert_eq!(extract_text(&val).unwrap(), None);
    }

    #[test]
    fn test_normalize_index_positive() {
        assert_eq!(normalize_index(2, 10), 2);
        assert_eq!(normalize_index(15, 10), 10);
    }

    #[test]
    fn test_normalize_index_negative() {
        assert_eq!(normalize_index(-1, 10), 9);
        assert_eq!(normalize_index(-3, 10), 7);
        assert_eq!(normalize_index(-15, 10), 0);
    }

    #[test]
    fn test_get_regex() {
        let re = get_regex(r"\d+").unwrap();
        assert!(re.is_match("123"));
    }

    #[test]
    fn test_get_regex_invalid() {
        let result = get_regex(r"[invalid");
        assert!(result.is_err());
    }
}
