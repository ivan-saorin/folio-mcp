//! Text transformation functions: case conversion, trimming, padding, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_text, require_text, extract_int_or, extract_optional_text};

// ============ Upper ============

pub struct Upper;

static UPPER_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to convert",
    optional: false,
    default: None,
}];

static UPPER_EXAMPLES: [&str; 2] = [
    "upper(\"hello\") → \"HELLO\"",
    "upper(\"café\") → \"CAFÉ\"",
];

static UPPER_RELATED: [&str; 3] = ["lower", "capitalize", "title_case"];

impl FunctionPlugin for Upper {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "upper",
            description: "Convert text to uppercase",
            usage: "upper(text)",
            args: &UPPER_ARGS,
            returns: "Text",
            examples: &UPPER_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &UPPER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("upper", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Text(s.to_uppercase()),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Lower ============

pub struct Lower;

static LOWER_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to convert",
    optional: false,
    default: None,
}];

static LOWER_EXAMPLES: [&str; 2] = [
    "lower(\"HELLO\") → \"hello\"",
    "lower(\"CAFÉ\") → \"café\"",
];

static LOWER_RELATED: [&str; 3] = ["upper", "capitalize", "title_case"];

impl FunctionPlugin for Lower {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lower",
            description: "Convert text to lowercase",
            usage: "lower(text)",
            args: &LOWER_ARGS,
            returns: "Text",
            examples: &LOWER_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &LOWER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("lower", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Text(s.to_lowercase()),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Capitalize ============

pub struct Capitalize;

static CAPITALIZE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to capitalize",
    optional: false,
    default: None,
}];

static CAPITALIZE_EXAMPLES: [&str; 1] = [
    "capitalize(\"hello world\") → \"Hello world\"",
];

static CAPITALIZE_RELATED: [&str; 2] = ["title_case", "upper"];

impl FunctionPlugin for Capitalize {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "capitalize",
            description: "Capitalize first character of text",
            usage: "capitalize(text)",
            args: &CAPITALIZE_ARGS,
            returns: "Text",
            examples: &CAPITALIZE_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &CAPITALIZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("capitalize", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => {
                if s.is_empty() {
                    return Value::Text(String::new());
                }
                let mut chars = s.chars();
                let first = chars.next().unwrap().to_uppercase().to_string();
                let rest: String = chars.collect();
                Value::Text(first + &rest)
            }
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ TitleCase ============

pub struct TitleCase;

static TITLE_CASE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to convert",
    optional: false,
    default: None,
}];

static TITLE_CASE_EXAMPLES: [&str; 1] = [
    "title_case(\"hello world\") → \"Hello World\"",
];

static TITLE_CASE_RELATED: [&str; 2] = ["capitalize", "upper"];

impl FunctionPlugin for TitleCase {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "title_case",
            description: "Capitalize first letter of each word",
            usage: "title_case(text)",
            args: &TITLE_CASE_ARGS,
            returns: "Text",
            examples: &TITLE_CASE_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &TITLE_CASE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("title_case", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => {
                let mut result = String::with_capacity(s.len());
                let mut capitalize_next = true;

                for c in s.chars() {
                    if c.is_whitespace() {
                        result.push(c);
                        capitalize_next = true;
                    } else if capitalize_next {
                        result.extend(c.to_uppercase());
                        capitalize_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                }

                Value::Text(result)
            }
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Trim ============

pub struct Trim;

static TRIM_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to trim",
    optional: false,
    default: None,
}];

static TRIM_EXAMPLES: [&str; 1] = [
    "trim(\"  hello  \") → \"hello\"",
];

static TRIM_RELATED: [&str; 3] = ["ltrim", "rtrim", "trim_chars"];

impl FunctionPlugin for Trim {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "trim",
            description: "Remove leading and trailing whitespace",
            usage: "trim(text)",
            args: &TRIM_ARGS,
            returns: "Text",
            examples: &TRIM_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &TRIM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("trim", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Text(s.trim().to_string()),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Ltrim ============

pub struct Ltrim;

static LTRIM_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to trim",
    optional: false,
    default: None,
}];

static LTRIM_EXAMPLES: [&str; 1] = [
    "ltrim(\"  hello  \") → \"hello  \"",
];

static LTRIM_RELATED: [&str; 2] = ["trim", "rtrim"];

impl FunctionPlugin for Ltrim {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ltrim",
            description: "Remove leading whitespace",
            usage: "ltrim(text)",
            args: &LTRIM_ARGS,
            returns: "Text",
            examples: &LTRIM_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &LTRIM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("ltrim", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Text(s.trim_start().to_string()),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Rtrim ============

pub struct Rtrim;

static RTRIM_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to trim",
    optional: false,
    default: None,
}];

static RTRIM_EXAMPLES: [&str; 1] = [
    "rtrim(\"  hello  \") → \"  hello\"",
];

static RTRIM_RELATED: [&str; 2] = ["trim", "ltrim"];

impl FunctionPlugin for Rtrim {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "rtrim",
            description: "Remove trailing whitespace",
            usage: "rtrim(text)",
            args: &RTRIM_ARGS,
            returns: "Text",
            examples: &RTRIM_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &RTRIM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("rtrim", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Text(s.trim_end().to_string()),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ TrimChars ============

pub struct TrimChars;

static TRIM_CHARS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to trim",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "chars",
        typ: "Text",
        description: "Characters to remove from both ends",
        optional: false,
        default: None,
    },
];

static TRIM_CHARS_EXAMPLES: [&str; 2] = [
    "trim_chars(\"...hello...\", \".\") → \"hello\"",
    "trim_chars(\"##test##\", \"#\") → \"test\"",
];

static TRIM_CHARS_RELATED: [&str; 1] = ["trim"];

impl FunctionPlugin for TrimChars {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "trim_chars",
            description: "Remove specific characters from both ends",
            usage: "trim_chars(text, chars)",
            args: &TRIM_CHARS_ARGS,
            returns: "Text",
            examples: &TRIM_CHARS_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &TRIM_CHARS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("trim_chars", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let chars = match require_text(&args[1], "trim_chars", "chars") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let char_set: Vec<char> = chars.chars().collect();
        let result = text
            .trim_start_matches(|c| char_set.contains(&c))
            .trim_end_matches(|c| char_set.contains(&c));

        Value::Text(result.to_string())
    }
}

// ============ PadLeft ============

pub struct PadLeft;

static PAD_LEFT_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to pad",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "length",
        typ: "Number",
        description: "Target length",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "char",
        typ: "Text",
        description: "Padding character (default: space)",
        optional: true,
        default: Some(" "),
    },
];

static PAD_LEFT_EXAMPLES: [&str; 2] = [
    "pad_left(\"42\", 5, \"0\") → \"00042\"",
    "pad_left(\"hi\", 5) → \"   hi\"",
];

static PAD_LEFT_RELATED: [&str; 2] = ["pad_right", "center"];

impl FunctionPlugin for PadLeft {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pad_left",
            description: "Pad text on left to reach target length",
            usage: "pad_left(text, length, [char])",
            args: &PAD_LEFT_ARGS,
            returns: "Text",
            examples: &PAD_LEFT_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &PAD_LEFT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("pad_left", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let length = match extract_int_or(&args[1..2], 0, 0, "pad_left", "length") {
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let pad_char = extract_optional_text(args, 2)
            .and_then(|s| s.chars().next())
            .unwrap_or(' ');

        let text_len = text.chars().count();
        if text_len >= length {
            return Value::Text(text.to_string());
        }

        let padding: String = std::iter::repeat(pad_char).take(length - text_len).collect();
        Value::Text(padding + text)
    }
}

// ============ PadRight ============

pub struct PadRight;

static PAD_RIGHT_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to pad",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "length",
        typ: "Number",
        description: "Target length",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "char",
        typ: "Text",
        description: "Padding character (default: space)",
        optional: true,
        default: Some(" "),
    },
];

static PAD_RIGHT_EXAMPLES: [&str; 1] = [
    "pad_right(\"42\", 5, \"0\") → \"42000\"",
];

static PAD_RIGHT_RELATED: [&str; 2] = ["pad_left", "center"];

impl FunctionPlugin for PadRight {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pad_right",
            description: "Pad text on right to reach target length",
            usage: "pad_right(text, length, [char])",
            args: &PAD_RIGHT_ARGS,
            returns: "Text",
            examples: &PAD_RIGHT_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &PAD_RIGHT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("pad_right", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let length = match extract_int_or(&args[1..2], 0, 0, "pad_right", "length") {
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let pad_char = extract_optional_text(args, 2)
            .and_then(|s| s.chars().next())
            .unwrap_or(' ');

        let text_len = text.chars().count();
        if text_len >= length {
            return Value::Text(text.to_string());
        }

        let padding: String = std::iter::repeat(pad_char).take(length - text_len).collect();
        Value::Text(text.to_string() + &padding)
    }
}

// ============ Center ============

pub struct Center;

static CENTER_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to center",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "length",
        typ: "Number",
        description: "Target length",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "char",
        typ: "Text",
        description: "Padding character (default: space)",
        optional: true,
        default: Some(" "),
    },
];

static CENTER_EXAMPLES: [&str; 1] = [
    "center(\"hi\", 6) → \"  hi  \"",
];

static CENTER_RELATED: [&str; 2] = ["pad_left", "pad_right"];

impl FunctionPlugin for Center {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "center",
            description: "Center text with padding on both sides",
            usage: "center(text, length, [char])",
            args: &CENTER_ARGS,
            returns: "Text",
            examples: &CENTER_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &CENTER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("center", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let length = match extract_int_or(&args[1..2], 0, 0, "center", "length") {
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let pad_char = extract_optional_text(args, 2)
            .and_then(|s| s.chars().next())
            .unwrap_or(' ');

        let text_len = text.chars().count();
        if text_len >= length {
            return Value::Text(text.to_string());
        }

        let total_padding = length - text_len;
        let left_padding = total_padding / 2;
        let right_padding = total_padding - left_padding;

        let left: String = std::iter::repeat(pad_char).take(left_padding).collect();
        let right: String = std::iter::repeat(pad_char).take(right_padding).collect();

        Value::Text(left + text + &right)
    }
}

// ============ Repeat ============

pub struct Repeat;

static REPEAT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to repeat",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of repetitions",
        optional: false,
        default: None,
    },
];

static REPEAT_EXAMPLES: [&str; 1] = [
    "repeat(\"ab\", 3) → \"ababab\"",
];

static REPEAT_RELATED: [&str; 1] = ["concat"];

impl FunctionPlugin for Repeat {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "repeat",
            description: "Repeat text n times",
            usage: "repeat(text, count)",
            args: &REPEAT_ARGS,
            returns: "Text",
            examples: &REPEAT_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &REPEAT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("repeat", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let count = match extract_int_or(&args[1..2], 0, 0, "repeat", "count") {
            Ok(n) if n < 0 => return Value::Error(FolioError::domain_error(
                "repeat() count must be non-negative"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        // Prevent excessive memory usage
        if count > 1_000_000 || text.len() * count > 100_000_000 {
            return Value::Error(FolioError::domain_error(
                "repeat() would create string too large"
            ));
        }

        Value::Text(text.repeat(count))
    }
}

// ============ Reverse ============

pub struct Reverse;

static REVERSE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to reverse",
    optional: false,
    default: None,
}];

static REVERSE_EXAMPLES: [&str; 1] = [
    "reverse(\"hello\") → \"olleh\"",
];

static REVERSE_RELATED: [&str; 0] = [];

impl FunctionPlugin for Reverse {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "reverse",
            description: "Reverse string (Unicode-aware)",
            usage: "reverse(text)",
            args: &REVERSE_ARGS,
            returns: "Text",
            examples: &REVERSE_EXAMPLES,
            category: "text/transform",
            source: None,
            related: &REVERSE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("reverse", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => {
                let reversed: String = s.chars().rev().collect();
                Value::Text(reversed)
            }
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_upper() {
        let f = Upper;
        let args = vec![Value::Text("hello".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "HELLO");
    }

    #[test]
    fn test_lower() {
        let f = Lower;
        let args = vec![Value::Text("HELLO".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "hello");
    }

    #[test]
    fn test_capitalize() {
        let f = Capitalize;
        let args = vec![Value::Text("hello world".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "Hello world");
    }

    #[test]
    fn test_title_case() {
        let f = TitleCase;
        let args = vec![Value::Text("hello world".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "Hello World");
    }

    #[test]
    fn test_trim() {
        let f = Trim;
        let args = vec![Value::Text("  hello  ".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "hello");
    }

    #[test]
    fn test_pad_left() {
        let f = PadLeft;
        let args = vec![
            Value::Text("42".to_string()),
            Value::Number(Number::from_i64(5)),
            Value::Text("0".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "00042");
    }

    #[test]
    fn test_repeat() {
        let f = Repeat;
        let args = vec![
            Value::Text("ab".to_string()),
            Value::Number(Number::from_i64(3)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "ababab");
    }

    #[test]
    fn test_reverse() {
        let f = Reverse;
        let args = vec![Value::Text("hello".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "olleh");
    }

    #[test]
    fn test_null_propagation() {
        let f = Upper;
        let args = vec![Value::Null];
        let result = f.call(&args, &eval_ctx());
        assert!(result.is_null());
    }
}
