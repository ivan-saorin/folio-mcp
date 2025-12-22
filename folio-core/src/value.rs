//! Runtime values in Folio
//!
//! Values can be numbers, text, booleans, objects (for DECOMPOSE results),
//! lists, null, or errors. Errors propagate through computations.

use crate::{Number, FolioError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime value in Folio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Number(Number),
    Text(String),
    Bool(bool),
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Null,
    Error(FolioError),
}

impl Value {
    // ========== Safe Accessors (never panic) ==========
    
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }
    
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }
    
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }
    
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }
    
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    
    // ========== Object Field Access ==========
    
    /// Get field from object. Returns Error value if not found or not an object.
    pub fn get(&self, key: &str) -> Value {
        match self {
            Value::Object(map) => {
                map.get(key).cloned().unwrap_or_else(|| {
                    Value::Error(FolioError::undefined_field(key))
                })
            }
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("Object", self.type_name())),
        }
    }
    
    /// Type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "Number",
            Value::Text(_) => "Text",
            Value::Bool(_) => "Bool",
            Value::Object(_) => "Object",
            Value::List(_) => "List",
            Value::Null => "Null",
            Value::Error(_) => "Error",
        }
    }
    
    // ========== Type Coercion ==========
    
    /// Convert to number (may return Error)
    pub fn to_number(&self) -> Value {
        match self {
            Value::Number(n) => Value::Number(n.clone()),
            Value::Text(s) => {
                match Number::from_str(s) {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(FolioError::from(e)),
                }
            }
            Value::Bool(b) => Value::Number(Number::from_i64(if *b { 1 } else { 0 })),
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("Number", self.type_name())),
        }
    }
    
    /// Convert to text (always succeeds)
    pub fn to_text(&self) -> Value {
        Value::Text(format!("{}", self))
    }
    
    /// Convert to bool (truthy/falsy)
    pub fn to_bool(&self) -> Value {
        match self {
            Value::Bool(b) => Value::Bool(*b),
            Value::Number(n) => Value::Bool(!n.is_zero()),
            Value::Text(s) => Value::Bool(!s.is_empty()),
            Value::Null => Value::Bool(false),
            Value::List(l) => Value::Bool(!l.is_empty()),
            Value::Object(o) => Value::Bool(!o.is_empty()),
            Value::Error(e) => Value::Error(e.clone()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Object(_) => write!(f, "[Object]"),
            Value::List(l) => write!(f, "[List({})]", l.len()),
            Value::Null => write!(f, "null"),
            Value::Error(e) => write!(f, "#ERROR: {}", e.code),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

// From implementations for convenience
impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(Number::from_i64(n))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Number> for Value {
    fn from(n: Number) -> Self {
        Value::Number(n)
    }
}
