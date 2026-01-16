//! Runtime values in Folio
//!
//! Values can be numbers, text, booleans, datetime, duration, objects
//! (for DECOMPOSE results), lists, null, or errors. Errors propagate
//! through computations.

use crate::{Number, FolioError, FolioDateTime, FolioDuration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime value in Folio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Number(Number),
    Text(String),
    Bool(bool),
    DateTime(FolioDateTime),
    Duration(FolioDuration),
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

    pub fn as_datetime(&self) -> Option<&FolioDateTime> {
        match self {
            Value::DateTime(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_duration(&self) -> Option<&FolioDuration> {
        match self {
            Value::Duration(d) => Some(d),
            _ => None,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_datetime(&self) -> bool {
        matches!(self, Value::DateTime(_))
    }

    pub fn is_duration(&self) -> bool {
        matches!(self, Value::Duration(_))
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
            Value::DateTime(_) => "DateTime",
            Value::Duration(_) => "Duration",
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
            Value::DateTime(_) => Value::Bool(true), // DateTime is always truthy
            Value::Duration(d) => Value::Bool(!d.is_zero()),
            Value::Null => Value::Bool(false),
            Value::List(l) => Value::Bool(!l.is_empty()),
            Value::Object(o) => Value::Bool(!o.is_empty()),
            Value::Error(e) => Value::Error(e.clone()),
        }
    }

    /// Convert to datetime (may return Error)
    pub fn to_datetime(&self) -> Value {
        match self {
            Value::DateTime(dt) => Value::DateTime(dt.clone()),
            Value::Text(s) => {
                match FolioDateTime::parse(s) {
                    Ok(dt) => Value::DateTime(dt),
                    Err(e) => Value::Error(FolioError::parse_error(format!("{}", e))),
                }
            }
            Value::Number(n) => {
                // Interpret as Unix timestamp in seconds
                if let Some(secs) = n.to_i64() {
                    Value::DateTime(FolioDateTime::from_unix_secs(secs))
                } else {
                    Value::Error(FolioError::type_error("DateTime", "Number (out of range)"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("DateTime", self.type_name())),
        }
    }

    /// Convert to duration (may return Error)
    pub fn to_duration(&self) -> Value {
        match self {
            Value::Duration(d) => Value::Duration(d.clone()),
            Value::Number(n) => {
                // Interpret as seconds
                if let Some(secs) = n.to_i64() {
                    Value::Duration(FolioDuration::from_secs(secs))
                } else {
                    Value::Error(FolioError::type_error("Duration", "Number (out of range)"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("Duration", self.type_name())),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::DateTime(dt) => write!(f, "{}", dt),
            Value::Duration(d) => write!(f, "{}", d),
            Value::Object(obj) => {
                // Smart object display based on type
                if let Some(Value::Text(t)) = obj.get("type") {
                    match t.as_str() {
                        "Matrix" => {
                            if let Some(Value::List(data)) = obj.get("data") {
                                let rows: Vec<String> = data.iter().map(|row| {
                                    if let Value::List(cols) = row {
                                        let vals: Vec<String> = cols.iter().map(|v| {
                                            if let Value::Number(n) = v { n.as_decimal(4) } else { v.to_string() }
                                        }).collect();
                                        format!("[{}]", vals.join(", "))
                                    } else { row.to_string() }
                                }).collect();
                                if rows.len() == 1 { write!(f, "{}", rows[0]) }
                                else { write!(f, "[{}]", rows.join("; ")) }
                            } else { write!(f, "[Matrix]") }
                        }
                        "Vector" => {
                            if let Some(Value::List(data)) = obj.get("data") {
                                let vals: Vec<String> = data.iter().map(|v| {
                                    if let Value::Number(n) = v { n.as_decimal(4) } else { v.to_string() }
                                }).collect();
                                write!(f, "[{}]", vals.join(", "))
                            } else { write!(f, "[Vector]") }
                        }
                        _ => write!(f, "[{}]", t)
                    }
                } else { write!(f, "[Object]") }
            }
            Value::List(items) => {
                // Smart list display: show values for small lists, count for large
                if items.len() <= 5 {
                    let contents: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                    write!(f, "[{}]", contents.join(", "))
                } else {
                    write!(f, "[{}]", items.len())
                }
            }
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

impl From<FolioDateTime> for Value {
    fn from(dt: FolioDateTime) -> Self {
        Value::DateTime(dt)
    }
}

impl From<FolioDuration> for Value {
    fn from(d: FolioDuration) -> Self {
        Value::Duration(d)
    }
}
