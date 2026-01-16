//! Text search functions: contains, index_of, matches, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_text, require_text, extract_int_or, get_regex};

// ============ Contains ============

pub struct Contains;

static CONTAINS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search in",
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
];

static CONTAINS_EXAMPLES: [&str; 2] = [
    "contains(\"hello world\", \"world\") → true",
    "contains(\"hello\", \"World\") → false",
];

static CONTAINS_RELATED: [&str; 3] = ["starts_with", "ends_with", "index_of"];

impl FunctionPlugin for Contains {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "contains",
            description: "Check if text contains substring (case-sensitive)",
            usage: "contains(text, search)",
            args: &CONTAINS_ARGS,
            returns: "Bool",
            examples: &CONTAINS_EXAMPLES,
            category: "text/search",
            source: None,
            related: &CONTAINS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("contains", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "contains", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        Value::Bool(text.contains(search))
    }
}

// ============ ContainsAny ============

pub struct ContainsAny;

static CONTAINS_ANY_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search in",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "searches",
        typ: "List<Text>",
        description: "List of substrings to find",
        optional: false,
        default: None,
    },
];

static CONTAINS_ANY_EXAMPLES: [&str; 1] = [
    "contains_any(\"hello\", [\"hi\", \"lo\"]) → true",
];

static CONTAINS_ANY_RELATED: [&str; 1] = ["contains"];

impl FunctionPlugin for ContainsAny {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "contains_any",
            description: "Check if text contains any of the substrings",
            usage: "contains_any(text, searches)",
            args: &CONTAINS_ANY_ARGS,
            returns: "Bool",
            examples: &CONTAINS_ANY_EXAMPLES,
            category: "text/search",
            source: None,
            related: &CONTAINS_ANY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("contains_any", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let searches = match &args[1] {
            Value::List(list) => list,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "contains_any", "searches", "List", other.type_name()
            )),
        };

        for item in searches {
            match item {
                Value::Text(s) => {
                    if text.contains(s.as_str()) {
                        return Value::Bool(true);
                    }
                }
                Value::Error(e) => return Value::Error(e.clone()),
                _ => continue, // Skip non-text items
            }
        }

        Value::Bool(false)
    }
}

// ============ StartsWith ============

pub struct StartsWith;

static STARTS_WITH_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to check",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "prefix",
        typ: "Text",
        description: "Prefix to find",
        optional: false,
        default: None,
    },
];

static STARTS_WITH_EXAMPLES: [&str; 1] = [
    "starts_with(\"hello\", \"he\") → true",
];

static STARTS_WITH_RELATED: [&str; 2] = ["ends_with", "contains"];

impl FunctionPlugin for StartsWith {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "starts_with",
            description: "Check if text starts with prefix",
            usage: "starts_with(text, prefix)",
            args: &STARTS_WITH_ARGS,
            returns: "Bool",
            examples: &STARTS_WITH_EXAMPLES,
            category: "text/search",
            source: None,
            related: &STARTS_WITH_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("starts_with", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let prefix = match require_text(&args[1], "starts_with", "prefix") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        Value::Bool(text.starts_with(prefix))
    }
}

// ============ EndsWith ============

pub struct EndsWith;

static ENDS_WITH_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to check",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "suffix",
        typ: "Text",
        description: "Suffix to find",
        optional: false,
        default: None,
    },
];

static ENDS_WITH_EXAMPLES: [&str; 1] = [
    "ends_with(\"hello\", \"lo\") → true",
];

static ENDS_WITH_RELATED: [&str; 2] = ["starts_with", "contains"];

impl FunctionPlugin for EndsWith {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ends_with",
            description: "Check if text ends with suffix",
            usage: "ends_with(text, suffix)",
            args: &ENDS_WITH_ARGS,
            returns: "Bool",
            examples: &ENDS_WITH_EXAMPLES,
            category: "text/search",
            source: None,
            related: &ENDS_WITH_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("ends_with", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let suffix = match require_text(&args[1], "ends_with", "suffix") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        Value::Bool(text.ends_with(suffix))
    }
}

// ============ IndexOf ============

pub struct IndexOf;

static INDEX_OF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search in",
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
        name: "start",
        typ: "Number",
        description: "Starting position (0-indexed)",
        optional: true,
        default: Some("0"),
    },
];

static INDEX_OF_EXAMPLES: [&str; 3] = [
    "index_of(\"hello\", \"l\") → 2",
    "index_of(\"hello\", \"l\", 3) → 3",
    "index_of(\"hello\", \"x\") → -1",
];

static INDEX_OF_RELATED: [&str; 2] = ["last_index_of", "contains"];

impl FunctionPlugin for IndexOf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "index_of",
            description: "Find first occurrence (0-indexed). Returns -1 if not found",
            usage: "index_of(text, search, [start])",
            args: &INDEX_OF_ARGS,
            returns: "Number",
            examples: &INDEX_OF_EXAMPLES,
            category: "text/search",
            source: None,
            related: &INDEX_OF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("index_of", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "index_of", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_int_or(args, 2, 0, "index_of", "start") {
            Ok(n) => n.max(0) as usize,
            Err(e) => return Value::Error(e),
        };

        // Work with characters for Unicode support
        let chars: Vec<char> = text.chars().collect();
        let search_chars: Vec<char> = search.chars().collect();

        if search_chars.is_empty() {
            return Value::Number(Number::from_i64(start as i64));
        }

        if start >= chars.len() {
            return Value::Number(Number::from_i64(-1));
        }

        // Search for the substring
        for i in start..=chars.len().saturating_sub(search_chars.len()) {
            let mut found = true;
            for (j, &sc) in search_chars.iter().enumerate() {
                if chars[i + j] != sc {
                    found = false;
                    break;
                }
            }
            if found {
                return Value::Number(Number::from_i64(i as i64));
            }
        }

        Value::Number(Number::from_i64(-1))
    }
}

// ============ LastIndexOf ============

pub struct LastIndexOf;

static LAST_INDEX_OF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search in",
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
];

static LAST_INDEX_OF_EXAMPLES: [&str; 1] = [
    "last_index_of(\"hello\", \"l\") → 3",
];

static LAST_INDEX_OF_RELATED: [&str; 2] = ["index_of", "contains"];

impl FunctionPlugin for LastIndexOf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "last_index_of",
            description: "Find last occurrence (0-indexed). Returns -1 if not found",
            usage: "last_index_of(text, search)",
            args: &LAST_INDEX_OF_ARGS,
            returns: "Number",
            examples: &LAST_INDEX_OF_EXAMPLES,
            category: "text/search",
            source: None,
            related: &LAST_INDEX_OF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("last_index_of", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "last_index_of", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let chars: Vec<char> = text.chars().collect();
        let search_chars: Vec<char> = search.chars().collect();

        if search_chars.is_empty() {
            return Value::Number(Number::from_i64(chars.len() as i64));
        }

        if chars.len() < search_chars.len() {
            return Value::Number(Number::from_i64(-1));
        }

        // Search backwards
        for i in (0..=chars.len() - search_chars.len()).rev() {
            let mut found = true;
            for (j, &sc) in search_chars.iter().enumerate() {
                if chars[i + j] != sc {
                    found = false;
                    break;
                }
            }
            if found {
                return Value::Number(Number::from_i64(i as i64));
            }
        }

        Value::Number(Number::from_i64(-1))
    }
}

// ============ CountMatches ============

pub struct CountMatches;

static COUNT_MATCHES_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search in",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "search",
        typ: "Text",
        description: "Substring to count",
        optional: false,
        default: None,
    },
];

static COUNT_MATCHES_EXAMPLES: [&str; 2] = [
    "count_matches(\"banana\", \"a\") → 3",
    "count_matches(\"aaa\", \"aa\") → 1",
];

static COUNT_MATCHES_RELATED: [&str; 2] = ["contains", "index_of"];

impl FunctionPlugin for CountMatches {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "count_matches",
            description: "Count non-overlapping occurrences of substring",
            usage: "count_matches(text, search)",
            args: &COUNT_MATCHES_ARGS,
            returns: "Number",
            examples: &COUNT_MATCHES_EXAMPLES,
            category: "text/search",
            source: None,
            related: &COUNT_MATCHES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("count_matches", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let search = match require_text(&args[1], "count_matches", "search") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        if search.is_empty() {
            return Value::Error(FolioError::domain_error(
                "count_matches() search string cannot be empty"
            ));
        }

        let count = text.matches(search).count();
        Value::Number(Number::from_i64(count as i64))
    }
}

// ============ Matches ============

pub struct Matches;

static MATCHES_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to test",
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
];

static MATCHES_EXAMPLES: [&str; 1] = [
    "matches(\"hello123\", r\"[a-z]+\\d+\") → true",
];

static MATCHES_RELATED: [&str; 2] = ["contains", "extract"];

impl FunctionPlugin for Matches {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "matches",
            description: "Check if text matches regex pattern",
            usage: "matches(text, pattern)",
            args: &MATCHES_ARGS,
            returns: "Bool",
            examples: &MATCHES_EXAMPLES,
            category: "text/search",
            source: None,
            related: &MATCHES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("matches", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "matches", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        Value::Bool(re.is_match(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_contains() {
        let f = Contains;
        let args = vec![
            Value::Text("hello world".to_string()),
            Value::Text("world".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_contains_false() {
        let f = Contains;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("World".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool().unwrap(), false);
    }

    #[test]
    fn test_starts_with() {
        let f = StartsWith;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("he".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_ends_with() {
        let f = EndsWith;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("lo".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool().unwrap(), true);
    }

    #[test]
    fn test_index_of() {
        let f = IndexOf;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("l".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_index_of_not_found() {
        let f = IndexOf;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("x".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(-1));
    }

    #[test]
    fn test_last_index_of() {
        let f = LastIndexOf;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text("l".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_count_matches() {
        let f = CountMatches;
        let args = vec![
            Value::Text("banana".to_string()),
            Value::Text("a".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_matches() {
        let f = Matches;
        let args = vec![
            Value::Text("hello123".to_string()),
            Value::Text(r"[a-z]+\d+".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool().unwrap(), true);
    }
}
