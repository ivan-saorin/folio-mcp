//! Text parsing functions: parse_number, parse_int, parse_float, parse_bool, parse_date, parse_json, parse_csv_line

use folio_plugin::prelude::*;
use crate::helpers::extract_text;
use std::collections::HashMap;

// ============ ParseNumber ============

pub struct ParseNumber;

static PARSE_NUMBER_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to parse as number",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "default",
        typ: "Number",
        description: "Default value if parsing fails",
        optional: true,
        default: None,
    },
];

static PARSE_NUMBER_EXAMPLES: [&str; 4] = [
    "parse_number(\"42\") → 42",
    "parse_number(\"3.14\") → 3.14",
    "parse_number(\"1,234.56\") → 1234.56",
    "parse_number(\"25%\") → 0.25",
];

static PARSE_NUMBER_RELATED: [&str; 2] = ["parse_int", "parse_float"];

impl FunctionPlugin for ParseNumber {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_number",
            description: "Parse text to number (handles integers, decimals, percentages, scientific notation)",
            usage: "parse_number(text, [default])",
            args: &PARSE_NUMBER_ARGS,
            returns: "Number",
            examples: &PARSE_NUMBER_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_NUMBER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_number", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let default_value = args.get(1).cloned();

        match parse_number_str(text) {
            Some(n) => Value::Number(n),
            None => match default_value {
                Some(Value::Number(n)) => Value::Number(n),
                Some(Value::Null) => Value::Null,
                Some(Value::Error(e)) => Value::Error(e),
                Some(_) => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as number",
                    text
                ))),
                None => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as number",
                    text
                ))),
            },
        }
    }
}

/// Parse a string as a number, handling various formats
fn parse_number_str(s: &str) -> Option<Number> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Handle percentage: "25%" -> 0.25
    if s.ends_with('%') {
        let num_str = s.trim_end_matches('%').trim();
        if let Some(n) = parse_simple_number(num_str) {
            return n.checked_div(&Number::from_i64(100)).ok();
        }
        return None;
    }

    parse_simple_number(s)
}

/// Parse a simple number (with optional thousands separators)
fn parse_simple_number(s: &str) -> Option<Number> {
    // Remove thousands separators (commas, spaces, underscores)
    let cleaned: String = s
        .chars()
        .filter(|c| *c != ',' && *c != '_' && *c != ' ')
        .collect();

    Number::from_str(&cleaned).ok()
}

// ============ ParseInt ============

pub struct ParseInt;

static PARSE_INT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to parse as integer",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "default",
        typ: "Number",
        description: "Default value if parsing fails",
        optional: true,
        default: None,
    },
];

static PARSE_INT_EXAMPLES: [&str; 2] = [
    "parse_int(\"42.9\") → 42",
    "parse_int(\"abc\", 0) → 0",
];

static PARSE_INT_RELATED: [&str; 2] = ["parse_number", "parse_float"];

impl FunctionPlugin for ParseInt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_int",
            description: "Parse text as integer (truncates decimal part)",
            usage: "parse_int(text, [default])",
            args: &PARSE_INT_ARGS,
            returns: "Number",
            examples: &PARSE_INT_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_INT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_int", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let default_value = args.get(1).cloned();

        match parse_number_str(text) {
            Some(n) => {
                // Truncate to integer (floor toward zero)
                match n.to_i64() {
                    Some(i) => Value::Number(Number::from_i64(i)),
                    None => {
                        // Large number: use floor for positive, ceil for negative
                        if n.is_negative() {
                            Value::Number(n.ceil())
                        } else {
                            Value::Number(n.floor())
                        }
                    }
                }
            }
            None => match default_value {
                Some(Value::Number(n)) => Value::Number(n),
                Some(Value::Null) => Value::Null,
                Some(Value::Error(e)) => Value::Error(e),
                Some(_) => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as integer",
                    text
                ))),
                None => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as integer",
                    text
                ))),
            },
        }
    }
}

// ============ ParseFloat ============

pub struct ParseFloat;

static PARSE_FLOAT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to parse as float",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "default",
        typ: "Number",
        description: "Default value if parsing fails",
        optional: true,
        default: None,
    },
];

static PARSE_FLOAT_EXAMPLES: [&str; 1] = ["parse_float(\"3.14159\") → 3.14159"];

static PARSE_FLOAT_RELATED: [&str; 2] = ["parse_number", "parse_int"];

impl FunctionPlugin for ParseFloat {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_float",
            description: "Parse text as floating-point number",
            usage: "parse_float(text, [default])",
            args: &PARSE_FLOAT_ARGS,
            returns: "Number",
            examples: &PARSE_FLOAT_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_FLOAT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_float", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let default_value = args.get(1).cloned();

        match parse_number_str(text) {
            Some(n) => Value::Number(n),
            None => match default_value {
                Some(Value::Number(n)) => Value::Number(n),
                Some(Value::Null) => Value::Null,
                Some(Value::Error(e)) => Value::Error(e),
                Some(_) => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as float",
                    text
                ))),
                None => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as float",
                    text
                ))),
            },
        }
    }
}

// ============ ParseBool ============

pub struct ParseBool;

static PARSE_BOOL_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to parse as boolean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "default",
        typ: "Bool",
        description: "Default value if parsing fails",
        optional: true,
        default: None,
    },
];

static PARSE_BOOL_EXAMPLES: [&str; 3] = [
    "parse_bool(\"true\") → true",
    "parse_bool(\"yes\") → true",
    "parse_bool(\"0\") → false",
];

static PARSE_BOOL_RELATED: [&str; 1] = ["is_numeric"];

impl FunctionPlugin for ParseBool {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_bool",
            description: "Parse text as boolean (truthy: true, yes, y, 1, on, t; falsy: false, no, n, 0, off, f)",
            usage: "parse_bool(text, [default])",
            args: &PARSE_BOOL_ARGS,
            returns: "Bool",
            examples: &PARSE_BOOL_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_BOOL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_bool", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let default_value = args.get(1).cloned();
        let lower = text.trim().to_lowercase();

        // Truthy values
        if matches!(lower.as_str(), "true" | "yes" | "y" | "1" | "on" | "t") {
            return Value::Bool(true);
        }

        // Falsy values
        if matches!(lower.as_str(), "false" | "no" | "n" | "0" | "off" | "f") {
            return Value::Bool(false);
        }

        // Not recognized - use default or error
        match default_value {
            Some(Value::Bool(b)) => Value::Bool(b),
            Some(Value::Null) => Value::Null,
            Some(Value::Error(e)) => Value::Error(e),
            Some(_) => Value::Error(FolioError::parse_error(format!(
                "Cannot parse '{}' as boolean",
                text
            ))),
            None => Value::Error(FolioError::parse_error(format!(
                "Cannot parse '{}' as boolean",
                text
            ))),
        }
    }
}

// ============ ParseDate ============

pub struct ParseDate;

static PARSE_DATE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to parse as date",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "format",
        typ: "Text",
        description: "Format pattern (YYYY, MM, DD, HH, mm, ss)",
        optional: true,
        default: None,
    },
    ArgMeta {
        name: "default",
        typ: "DateTime",
        description: "Default value if parsing fails",
        optional: true,
        default: None,
    },
];

static PARSE_DATE_EXAMPLES: [&str; 3] = [
    "parse_date(\"2024-03-15\") → DateTime",
    "parse_date(\"15/03/2024\", \"DD/MM/YYYY\") → DateTime",
    "parse_date(\"March 15, 2024\") → DateTime",
];

static PARSE_DATE_RELATED: [&str; 1] = ["date"];

impl FunctionPlugin for ParseDate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_date",
            description: "Parse text to DateTime with optional format",
            usage: "parse_date(text, [format], [default])",
            args: &PARSE_DATE_ARGS,
            returns: "DateTime",
            examples: &PARSE_DATE_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_DATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_date", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        // Check if second arg is format or default
        let (format, default_value) = match args.get(1) {
            Some(Value::Text(s)) => (Some(s.as_str()), args.get(2).cloned()),
            Some(Value::DateTime(_)) => (None, args.get(1).cloned()),
            Some(Value::Null) => (None, args.get(2).cloned()),
            Some(Value::Error(e)) => return Value::Error(e.clone()),
            _ => (None, args.get(1).cloned()),
        };

        let parse_result = if let Some(fmt) = format {
            FolioDateTime::parse_format(text.trim(), fmt)
                .map_err(|e| FolioError::parse_error(e.to_string()))
        } else {
            // Auto-detect format - try various patterns
            parse_date_auto(text.trim())
        };

        match parse_result {
            Ok(dt) => Value::DateTime(dt),
            Err(_) => match default_value {
                Some(Value::DateTime(dt)) => Value::DateTime(dt),
                Some(Value::Null) => Value::Null,
                Some(Value::Error(e)) => Value::Error(e),
                Some(_) => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as date",
                    text
                ))),
                None => Value::Error(FolioError::parse_error(format!(
                    "Cannot parse '{}' as date",
                    text
                ))),
            },
        }
    }
}

/// Auto-detect date format and parse
fn parse_date_auto(s: &str) -> Result<FolioDateTime, FolioError> {
    // Try ISO 8601 first (most common)
    if let Ok(dt) = FolioDateTime::parse(s) {
        return Ok(dt);
    }

    // Try US format: MM/DD/YYYY or M/D/YYYY
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            // Try MM/DD/YYYY
            if let Ok(dt) = parse_mdy(&parts) {
                return Ok(dt);
            }
        }
    }

    // Try EU format: DD.MM.YYYY or DD-MM-YYYY
    if s.contains('.') {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() == 3 {
            if let Ok(dt) = parse_dmy(&parts) {
                return Ok(dt);
            }
        }
    }

    // Try long format: "March 15, 2024" or "15 March 2024"
    if let Ok(dt) = parse_long_date(s) {
        return Ok(dt);
    }

    Err(FolioError::parse_error(format!("Cannot parse '{}' as date", s)))
}

fn parse_mdy(parts: &[&str]) -> Result<FolioDateTime, FolioError> {
    let month: u32 = parts[0].parse().map_err(|_| FolioError::parse_error("Invalid month"))?;
    let day: u32 = parts[1].parse().map_err(|_| FolioError::parse_error("Invalid day"))?;
    let year: i32 = parse_year(parts[2])?;
    FolioDateTime::from_ymd(year, month, day)
        .map_err(|e| FolioError::parse_error(e.to_string()))
}

fn parse_dmy(parts: &[&str]) -> Result<FolioDateTime, FolioError> {
    let day: u32 = parts[0].parse().map_err(|_| FolioError::parse_error("Invalid day"))?;
    let month: u32 = parts[1].parse().map_err(|_| FolioError::parse_error("Invalid month"))?;
    let year: i32 = parse_year(parts[2])?;
    FolioDateTime::from_ymd(year, month, day)
        .map_err(|e| FolioError::parse_error(e.to_string()))
}

fn parse_year(s: &str) -> Result<i32, FolioError> {
    let year: i32 = s.parse().map_err(|_| FolioError::parse_error("Invalid year"))?;
    // Handle 2-digit years
    if year < 100 {
        if year >= 70 {
            Ok(1900 + year)
        } else {
            Ok(2000 + year)
        }
    } else {
        Ok(year)
    }
}

fn parse_long_date(s: &str) -> Result<FolioDateTime, FolioError> {
    let months: [(&str, u32); 12] = [
        ("january", 1), ("february", 2), ("march", 3), ("april", 4),
        ("may", 5), ("june", 6), ("july", 7), ("august", 8),
        ("september", 9), ("october", 10), ("november", 11), ("december", 12),
    ];
    let short_months: [(&str, u32); 12] = [
        ("jan", 1), ("feb", 2), ("mar", 3), ("apr", 4),
        ("may", 5), ("jun", 6), ("jul", 7), ("aug", 8),
        ("sep", 9), ("oct", 10), ("nov", 11), ("dec", 12),
    ];

    let lower = s.to_lowercase();
    let clean: String = lower.replace(',', " ").replace('.', " ");
    let parts: Vec<&str> = clean.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(FolioError::parse_error("Invalid date format"));
    }

    // Find month
    let mut month = None;
    let mut month_pos = None;
    for (i, part) in parts.iter().enumerate() {
        for (name, m) in &months {
            if part.starts_with(*name) || *part == *name {
                month = Some(*m);
                month_pos = Some(i);
                break;
            }
        }
        if month.is_none() {
            for (name, m) in &short_months {
                if part.starts_with(*name) {
                    month = Some(*m);
                    month_pos = Some(i);
                    break;
                }
            }
        }
        if month.is_some() {
            break;
        }
    }

    let month = month.ok_or_else(|| FolioError::parse_error("Month not found"))?;
    let month_pos = month_pos.unwrap();

    // Find day and year in remaining parts
    let mut day = None;
    let mut year = None;

    for (i, part) in parts.iter().enumerate() {
        if i == month_pos {
            continue;
        }
        if let Ok(n) = part.trim_end_matches(|c: char| !c.is_ascii_digit()).parse::<i32>() {
            if n > 31 || (year.is_none() && n > 2000) {
                year = Some(n);
            } else if day.is_none() && n >= 1 && n <= 31 {
                day = Some(n as u32);
            } else if year.is_none() {
                year = Some(if n < 100 { if n >= 70 { 1900 + n } else { 2000 + n } } else { n });
            }
        }
    }

    let day = day.ok_or_else(|| FolioError::parse_error("Day not found"))?;
    let year = year.ok_or_else(|| FolioError::parse_error("Year not found"))?;

    FolioDateTime::from_ymd(year, month, day)
        .map_err(|e| FolioError::parse_error(e.to_string()))
}

// ============ ParseJson ============

pub struct ParseJson;

static PARSE_JSON_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "JSON string to parse",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "default",
        typ: "Any",
        description: "Default value if parsing fails",
        optional: true,
        default: None,
    },
];

static PARSE_JSON_EXAMPLES: [&str; 1] = [
    "parse_json('{\"a\": 1, \"b\": 2}') → {a: 1, b: 2}",
];

static PARSE_JSON_RELATED: [&str; 1] = ["parse_csv_line"];

impl FunctionPlugin for ParseJson {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_json",
            description: "Parse JSON string to Object/List/Value",
            usage: "parse_json(text, [default])",
            args: &PARSE_JSON_ARGS,
            returns: "Object | List | Number | Text | Bool | Null",
            examples: &PARSE_JSON_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_JSON_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_json", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let default_value = args.get(1).cloned();

        match parse_json_value(text) {
            Ok(v) => v,
            Err(_) => match default_value {
                Some(v) => v,
                None => Value::Error(FolioError::parse_error(format!(
                    "Invalid JSON: '{}'",
                    if text.len() > 50 { &text[..50] } else { text }
                ))),
            },
        }
    }
}

/// Simple JSON parser that converts to Folio Values
fn parse_json_value(s: &str) -> Result<Value, FolioError> {
    let s = s.trim();

    if s.is_empty() {
        return Err(FolioError::parse_error("Empty JSON"));
    }

    // Null
    if s == "null" {
        return Ok(Value::Null);
    }

    // Boolean
    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }

    // Number
    if s.starts_with('-') || s.starts_with(|c: char| c.is_ascii_digit()) {
        if let Ok(n) = Number::from_str(s) {
            return Ok(Value::Number(n));
        }
    }

    // String
    if s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        let unescaped = unescape_json_string(inner);
        return Ok(Value::Text(unescaped));
    }

    // Array
    if s.starts_with('[') && s.ends_with(']') {
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Value::List(Vec::new()));
        }
        let elements = split_json_elements(inner)?;
        let values: Result<Vec<Value>, FolioError> = elements
            .iter()
            .map(|e| parse_json_value(e))
            .collect();
        return Ok(Value::List(values?));
    }

    // Object
    if s.starts_with('{') && s.ends_with('}') {
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Value::Object(HashMap::new()));
        }
        let pairs = split_json_elements(inner)?;
        let mut obj = HashMap::new();
        for pair in pairs {
            let (key, value) = parse_json_pair(&pair)?;
            obj.insert(key, value);
        }
        return Ok(Value::Object(obj));
    }

    Err(FolioError::parse_error("Invalid JSON syntax"))
}

fn unescape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('/') => result.push('/'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(h) = chars.next() {
                            hex.push(h);
                        }
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn split_json_elements(s: &str) -> Result<Vec<String>, FolioError> {
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for c in s.chars() {
        if in_string {
            current.push(c);
            if c == '"' && prev_char != '\\' {
                in_string = false;
            }
        } else {
            match c {
                '"' => {
                    in_string = true;
                    current.push(c);
                }
                '[' | '{' => {
                    depth += 1;
                    current.push(c);
                }
                ']' | '}' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    elements.push(current.trim().to_string());
                    current = String::new();
                }
                _ => {
                    current.push(c);
                }
            }
        }
        prev_char = c;
    }

    if !current.trim().is_empty() {
        elements.push(current.trim().to_string());
    }

    Ok(elements)
}

fn parse_json_pair(s: &str) -> Result<(String, Value), FolioError> {
    let s = s.trim();

    // Find the colon that separates key and value
    let mut colon_pos = None;
    let mut in_string = false;
    let mut prev_char = ' ';

    for (i, c) in s.chars().enumerate() {
        if in_string {
            if c == '"' && prev_char != '\\' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == ':' {
            colon_pos = Some(i);
            break;
        }
        prev_char = c;
    }

    let colon_pos = colon_pos.ok_or_else(|| FolioError::parse_error("Missing colon in JSON pair"))?;

    let key_str = s[..colon_pos].trim();
    let value_str = s[colon_pos + 1..].trim();

    // Extract key (must be a string)
    let key = if key_str.starts_with('"') && key_str.ends_with('"') {
        unescape_json_string(&key_str[1..key_str.len() - 1])
    } else {
        // Allow unquoted keys for convenience
        key_str.to_string()
    };

    let value = parse_json_value(value_str)?;

    Ok((key, value))
}

// ============ ParseCsvLine ============

pub struct ParseCsvLine;

static PARSE_CSV_LINE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "CSV line to parse",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "delimiter",
        typ: "Text",
        description: "Field delimiter (default: comma)",
        optional: true,
        default: Some(","),
    },
];

static PARSE_CSV_LINE_EXAMPLES: [&str; 3] = [
    "parse_csv_line(\"a,b,c\") → [\"a\", \"b\", \"c\"]",
    "parse_csv_line('\"a,b\",c') → [\"a,b\", \"c\"]",
    "parse_csv_line(\"a;b;c\", \";\") → [\"a\", \"b\", \"c\"]",
];

static PARSE_CSV_LINE_RELATED: [&str; 2] = ["split", "parse_json"];

impl FunctionPlugin for ParseCsvLine {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parse_csv_line",
            description: "Parse a single CSV line into a list of values",
            usage: "parse_csv_line(text, [delimiter])",
            args: &PARSE_CSV_LINE_ARGS,
            returns: "List<Text>",
            examples: &PARSE_CSV_LINE_EXAMPLES,
            category: "text/parse",
            source: None,
            related: &PARSE_CSV_LINE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("parse_csv_line", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let delimiter = match args.get(1) {
            Some(Value::Text(s)) if !s.is_empty() => s.chars().next().unwrap(),
            Some(Value::Null) | None => ',',
            Some(Value::Error(e)) => return Value::Error(e.clone()),
            Some(_) => ',',
        };

        let fields = parse_csv_fields(text, delimiter);
        Value::List(fields.into_iter().map(Value::Text).collect())
    }
}

fn parse_csv_fields(s: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                // Check for escaped quote
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == delimiter {
            fields.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }

    // Don't forget the last field
    fields.push(current.trim().to_string());

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_parse_number_integer() {
        let f = ParseNumber;
        let args = vec![Value::Text("42".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(42));
    }

    #[test]
    fn test_parse_number_decimal() {
        let f = ParseNumber;
        let args = vec![Value::Text("3.14".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert!(result.as_number().is_some());
    }

    #[test]
    fn test_parse_number_thousands() {
        let f = ParseNumber;
        let args = vec![Value::Text("1,234,567".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1234567));
    }

    #[test]
    fn test_parse_number_percentage() {
        let f = ParseNumber;
        let args = vec![Value::Text("25%".to_string())];
        let result = f.call(&args, &eval_ctx());
        let n = result.as_number().unwrap();
        // 25% = 0.25
        assert_eq!(n.as_decimal(2), "0.25");
    }

    #[test]
    fn test_parse_number_default() {
        let f = ParseNumber;
        let args = vec![
            Value::Text("abc".to_string()),
            Value::Number(Number::from_i64(0)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(0));
    }

    #[test]
    fn test_parse_int() {
        let f = ParseInt;
        let args = vec![Value::Text("42.9".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(42));
    }

    #[test]
    fn test_parse_bool_true() {
        let f = ParseBool;
        for val in ["true", "yes", "1", "on", "y", "t"] {
            let args = vec![Value::Text(val.to_string())];
            let result = f.call(&args, &eval_ctx());
            assert_eq!(result.as_bool(), Some(true), "Failed for {}", val);
        }
    }

    #[test]
    fn test_parse_bool_false() {
        let f = ParseBool;
        for val in ["false", "no", "0", "off", "n", "f"] {
            let args = vec![Value::Text(val.to_string())];
            let result = f.call(&args, &eval_ctx());
            assert_eq!(result.as_bool(), Some(false), "Failed for {}", val);
        }
    }

    #[test]
    fn test_parse_date_iso() {
        let f = ParseDate;
        let args = vec![Value::Text("2024-03-15".to_string())];
        let result = f.call(&args, &eval_ctx());
        let dt = result.as_datetime().unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_json_object() {
        let f = ParseJson;
        let args = vec![Value::Text(r#"{"a": 1, "b": 2}"#.to_string())];
        let result = f.call(&args, &eval_ctx());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("b"));
    }

    #[test]
    fn test_parse_json_array() {
        let f = ParseJson;
        let args = vec![Value::Text("[1, 2, 3]".to_string())];
        let result = f.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_parse_csv_line() {
        let f = ParseCsvLine;
        let args = vec![Value::Text("a,b,c".to_string())];
        let result = f.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_text(), Some("a"));
        assert_eq!(list[1].as_text(), Some("b"));
        assert_eq!(list[2].as_text(), Some("c"));
    }

    #[test]
    fn test_parse_csv_line_quoted() {
        let f = ParseCsvLine;
        let args = vec![Value::Text(r#""a,b",c"#.to_string())];
        let result = f.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_text(), Some("a,b"));
        assert_eq!(list[1].as_text(), Some("c"));
    }

    #[test]
    fn test_parse_csv_line_custom_delimiter() {
        let f = ParseCsvLine;
        let args = vec![
            Value::Text("a;b;c".to_string()),
            Value::Text(";".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
    }
}
