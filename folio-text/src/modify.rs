//! Text modification functions: replace, remove, insert, truncate, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_text, require_text, extract_int_or, extract_optional_text, get_regex, normalize_index};

// ============ Replace ============

pub struct Replace;

static REPLACE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "search",
        typ: "Text",
        description: "Substring to find",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "replacement",
        typ: "Text",
        description: "Replacement string",
        optional: false,
        default: None,
    },
];

static REPLACE_EXAMPLES: [&str; 1] = [
    "replace(\"hello\", \"l\", \"L\") → \"heLlo\"",
];

static REPLACE_RELATED: [&str; 2] = ["replace_all", "replace_regex"];

impl FunctionPlugin for Replace {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "replace",
            description: "Replace first occurrence of substring",
            usage: "replace(text, search, replacement)",
            args: &REPLACE_ARGS,
            returns: "Text",
            examples: &REPLACE_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &REPLACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("replace", 3, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "replace", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let replacement = match require_text(&args[2], "replace", "replacement") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        Value::Text(text.replacen(search, replacement, 1))
    }
}

// ============ ReplaceAll ============

pub struct ReplaceAll;

static REPLACE_ALL_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "search",
        typ: "Text",
        description: "Substring to find",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "replacement",
        typ: "Text",
        description: "Replacement string",
        optional: false,
        default: None,
    },
];

static REPLACE_ALL_EXAMPLES: [&str; 1] = [
    "replace_all(\"hello\", \"l\", \"L\") → \"heLLo\"",
];

static REPLACE_ALL_RELATED: [&str; 2] = ["replace", "replace_regex"];

impl FunctionPlugin for ReplaceAll {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "replace_all",
            description: "Replace all occurrences of substring",
            usage: "replace_all(text, search, replacement)",
            args: &REPLACE_ALL_ARGS,
            returns: "Text",
            examples: &REPLACE_ALL_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &REPLACE_ALL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("replace_all", 3, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "replace_all", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let replacement = match require_text(&args[2], "replace_all", "replacement") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        Value::Text(text.replace(search, replacement))
    }
}

// ============ ReplaceRegex ============

pub struct ReplaceRegex;

static REPLACE_REGEX_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pattern",
        typ: "Text",
        description: "Regex pattern",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "replacement",
        typ: "Text",
        description: "Replacement (supports $1, $2 backreferences)",
        optional: false,
        default: None,
    },
];

static REPLACE_REGEX_EXAMPLES: [&str; 2] = [
    "replace_regex(\"hello123\", r\"\\d+\", \"XXX\") → \"helloXXX\"",
    "replace_regex(\"John Smith\", r\"(\\w+) (\\w+)\", \"$2, $1\") → \"Smith, John\"",
];

static REPLACE_REGEX_RELATED: [&str; 2] = ["replace", "replace_all"];

impl FunctionPlugin for ReplaceRegex {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "replace_regex",
            description: "Replace using regex pattern (supports $1, $2 backreferences)",
            usage: "replace_regex(text, pattern, replacement)",
            args: &REPLACE_REGEX_ARGS,
            returns: "Text",
            examples: &REPLACE_REGEX_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &REPLACE_REGEX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("replace_regex", 3, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "replace_regex", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let replacement = match require_text(&args[2], "replace_regex", "replacement") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        Value::Text(re.replace_all(text, replacement).to_string())
    }
}

// ============ Remove ============

pub struct Remove;

static REMOVE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "search",
        typ: "Text",
        description: "Substring to remove",
        optional: false,
        default: None,
    },
];

static REMOVE_EXAMPLES: [&str; 1] = [
    "remove(\"hello\", \"l\") → \"heo\"",
];

static REMOVE_RELATED: [&str; 2] = ["remove_regex", "replace_all"];

impl FunctionPlugin for Remove {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "remove",
            description: "Remove all occurrences of substring",
            usage: "remove(text, search)",
            args: &REMOVE_ARGS,
            returns: "Text",
            examples: &REMOVE_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &REMOVE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("remove", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "remove", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        Value::Text(text.replace(search, ""))
    }
}

// ============ RemoveRegex ============

pub struct RemoveRegex;

static REMOVE_REGEX_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pattern",
        typ: "Text",
        description: "Regex pattern to remove",
        optional: false,
        default: None,
    },
];

static REMOVE_REGEX_EXAMPLES: [&str; 1] = [
    "remove_regex(\"hello123world456\", r\"\\d+\") → \"helloworld\"",
];

static REMOVE_REGEX_RELATED: [&str; 2] = ["remove", "replace_regex"];

impl FunctionPlugin for RemoveRegex {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "remove_regex",
            description: "Remove all regex matches",
            usage: "remove_regex(text, pattern)",
            args: &REMOVE_REGEX_ARGS,
            returns: "Text",
            examples: &REMOVE_REGEX_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &REMOVE_REGEX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("remove_regex", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "remove_regex", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        Value::Text(re.replace_all(text, "").to_string())
    }
}

// ============ Insert ============

pub struct Insert;

static INSERT_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "index",
        typ: "Number",
        description: "Position to insert at (0-indexed)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "insertion",
        typ: "Text",
        description: "Text to insert",
        optional: false,
        default: None,
    },
];

static INSERT_EXAMPLES: [&str; 1] = [
    "insert(\"hello\", 2, \"XY\") → \"heXYllo\"",
];

static INSERT_RELATED: [&str; 1] = ["replace"];

impl FunctionPlugin for Insert {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "insert",
            description: "Insert text at position",
            usage: "insert(text, index, insertion)",
            args: &INSERT_ARGS,
            returns: "Text",
            examples: &INSERT_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &INSERT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("insert", 3, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let index = match extract_int_or(&args[1..2], 0, 0, "insert", "index") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let insertion = match require_text(&args[2], "insert", "insertion") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let chars: Vec<char> = text.chars().collect();
        let idx = normalize_index(index, chars.len());

        let mut result = String::with_capacity(text.len() + insertion.len());
        result.extend(chars[..idx].iter());
        result.push_str(insertion);
        result.extend(chars[idx..].iter());

        Value::Text(result)
    }
}

// ============ Truncate ============

pub struct Truncate;

static TRUNCATE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to truncate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "max_length",
        typ: "Number",
        description: "Maximum length",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "suffix",
        typ: "Text",
        description: "Suffix to append if truncated (default: \"\")",
        optional: true,
        default: Some("\"\""),
    },
];

static TRUNCATE_EXAMPLES: [&str; 2] = [
    "truncate(\"hello world\", 8, \"...\") → \"hello...\"",
    "truncate(\"hi\", 8, \"...\") → \"hi\"",
];

static TRUNCATE_RELATED: [&str; 1] = ["ellipsis"];

impl FunctionPlugin for Truncate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "truncate",
            description: "Truncate text to max length with optional suffix",
            usage: "truncate(text, max_length, [suffix])",
            args: &TRUNCATE_ARGS,
            returns: "Text",
            examples: &TRUNCATE_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &TRUNCATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("truncate", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let max_length = match extract_int_or(&args[1..2], 0, 0, "truncate", "max_length") {
            Ok(n) if n < 0 => return Value::Error(FolioError::domain_error(
                "truncate() max_length must be non-negative"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let suffix = extract_optional_text(args, 2).unwrap_or("");

        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= max_length {
            return Value::Text(text.to_string());
        }

        let suffix_len = suffix.chars().count();
        if suffix_len >= max_length {
            // Suffix is too long, just take max_length chars from suffix
            let result: String = suffix.chars().take(max_length).collect();
            return Value::Text(result);
        }

        let content_len = max_length - suffix_len;
        let mut result: String = chars[..content_len].iter().collect();
        result.push_str(suffix);

        Value::Text(result)
    }
}

// ============ Ellipsis ============

pub struct Ellipsis;

static ELLIPSIS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to truncate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "max_length",
        typ: "Number",
        description: "Maximum length including ellipsis",
        optional: false,
        default: None,
    },
];

static ELLIPSIS_EXAMPLES: [&str; 1] = [
    "ellipsis(\"hello beautiful world\", 15) → \"hello...\"",
];

static ELLIPSIS_RELATED: [&str; 1] = ["truncate"];

impl FunctionPlugin for Ellipsis {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ellipsis",
            description: "Truncate with ellipsis, word-aware when possible",
            usage: "ellipsis(text, max_length)",
            args: &ELLIPSIS_ARGS,
            returns: "Text",
            examples: &ELLIPSIS_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &ELLIPSIS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("ellipsis", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let max_length = match extract_int_or(&args[1..2], 0, 0, "ellipsis", "max_length") {
            Ok(n) if n < 4 => return Value::Error(FolioError::domain_error(
                "ellipsis() max_length must be at least 4 (for \"...\" + 1 char)"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= max_length {
            return Value::Text(text.to_string());
        }

        // Reserve 3 chars for "..."
        let content_len = max_length - 3;
        let content: String = chars[..content_len].iter().collect();

        // Try to break at word boundary
        if let Some(last_space) = content.rfind(' ') {
            if last_space > content_len / 2 {
                // Found a reasonable word boundary
                return Value::Text(format!("{}...", content[..last_space].trim_end()));
            }
        }

        // No good word boundary, just truncate
        Value::Text(format!("{}...", content.trim_end()))
    }
}

// ============ Squeeze ============

pub struct Squeeze;

static SQUEEZE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to squeeze",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "char",
        typ: "Text",
        description: "Character to squeeze (default: all characters)",
        optional: true,
        default: None,
    },
];

static SQUEEZE_EXAMPLES: [&str; 2] = [
    "squeeze(\"hellooo   world\") → \"helo world\"",
    "squeeze(\"hellooo   world\", \" \") → \"hellooo world\"",
];

static SQUEEZE_RELATED: [&str; 1] = ["trim"];

impl FunctionPlugin for Squeeze {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "squeeze",
            description: "Collapse consecutive duplicate characters to single",
            usage: "squeeze(text, [char])",
            args: &SQUEEZE_ARGS,
            returns: "Text",
            examples: &SQUEEZE_EXAMPLES,
            category: "text/modify",
            source: None,
            related: &SQUEEZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("squeeze", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let specific_char = extract_optional_text(args, 1)
            .and_then(|s| s.chars().next());

        let mut result = String::with_capacity(text.len());
        let mut prev_char: Option<char> = None;

        for c in text.chars() {
            let should_squeeze = match specific_char {
                Some(target) => c == target && prev_char == Some(c),
                None => prev_char == Some(c),
            };

            if !should_squeeze {
                result.push(c);
            }
            prev_char = Some(c);
        }

        Value::Text(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_replace() {
        let f = Replace;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("l".to_string()),
            Value::Text("L".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "heLlo");
    }

    #[test]
    fn test_replace_all() {
        let f = ReplaceAll;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("l".to_string()),
            Value::Text("L".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "heLLo");
    }

    #[test]
    fn test_remove() {
        let f = Remove;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("l".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "heo");
    }

    #[test]
    fn test_insert() {
        let f = Insert;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Number(Number::from_i64(2)),
            Value::Text("XY".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "heXYllo");
    }

    #[test]
    fn test_truncate() {
        let f = Truncate;
        let args = vec![
            Value::Text("hello world".to_string()),
            Value::Number(Number::from_i64(8)),
            Value::Text("...".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "hello...");
    }

    #[test]
    fn test_truncate_no_truncation() {
        let f = Truncate;
        let args = vec![
            Value::Text("hi".to_string()),
            Value::Number(Number::from_i64(8)),
            Value::Text("...".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "hi");
    }

    #[test]
    fn test_squeeze() {
        let f = Squeeze;
        let args = vec![
            Value::Text("hellooo   world".to_string()),
            Value::Text(" ".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "hellooo world");
    }
}
