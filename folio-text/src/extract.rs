//! Text extraction functions: len, substring, split, regex extract, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_text, require_text, extract_int_or, normalize_index, get_regex};

// ============ Len ============

pub struct Len;

static LEN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to measure",
    optional: false,
    default: None,
}];

static LEN_EXAMPLES: [&str; 3] = [
    "len(\"hello\") → 5",
    "len(\"café\") → 4",
    "len(\"日本\") → 2",
];

static LEN_RELATED: [&str; 1] = ["byte_len"];

impl FunctionPlugin for Len {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "len",
            description: "Length in characters (Unicode code points)",
            usage: "len(text)",
            args: &LEN_ARGS,
            returns: "Number",
            examples: &LEN_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &LEN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("len", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Number(Number::from_i64(s.chars().count() as i64)),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ ByteLen ============

pub struct ByteLen;

static BYTE_LEN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to measure",
    optional: false,
    default: None,
}];

static BYTE_LEN_EXAMPLES: [&str; 2] = [
    "byte_len(\"café\") → 5",
    "byte_len(\"日本\") → 6",
];

static BYTE_LEN_RELATED: [&str; 1] = ["len"];

impl FunctionPlugin for ByteLen {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "byte_len",
            description: "Length in bytes (UTF-8)",
            usage: "byte_len(text)",
            args: &BYTE_LEN_ARGS,
            returns: "Number",
            examples: &BYTE_LEN_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &BYTE_LEN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("byte_len", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Number(Number::from_i64(s.len() as i64)),
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ CharAt ============

pub struct CharAt;

static CHAR_AT_ARGS: [ArgMeta; 2] = [
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
        description: "Position (0-indexed, negative counts from end)",
        optional: false,
        default: None,
    },
];

static CHAR_AT_EXAMPLES: [&str; 1] = [
    "char_at(\"hello\", 1) → \"e\"",
];

static CHAR_AT_RELATED: [&str; 2] = ["substring", "left"];

impl FunctionPlugin for CharAt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "char_at",
            description: "Character at index (0-indexed)",
            usage: "char_at(text, index)",
            args: &CHAR_AT_ARGS,
            returns: "Text",
            examples: &CHAR_AT_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &CHAR_AT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("char_at", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let index = match extract_int_or(&args[1..2], 0, 0, "char_at", "index") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let chars: Vec<char> = text.chars().collect();
        let idx = normalize_index(index, chars.len());

        if idx >= chars.len() {
            return Value::Text(String::new());
        }

        Value::Text(chars[idx].to_string())
    }
}

// ============ Substring ============

pub struct Substring;

static SUBSTRING_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start position (0-indexed, negative counts from end)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "End position (exclusive, negative counts from end)",
        optional: true,
        default: None,
    },
];

static SUBSTRING_EXAMPLES: [&str; 4] = [
    "substring(\"hello\", 1, 4) → \"ell\"",
    "substring(\"hello\", 2) → \"llo\"",
    "substring(\"hello\", -3) → \"llo\"",
    "substring(\"hello\", 1, -1) → \"ell\"",
];

static SUBSTRING_RELATED: [&str; 3] = ["left", "right", "mid"];

impl FunctionPlugin for Substring {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "substring",
            description: "Extract substring (0-indexed, end exclusive, negative indices count from end)",
            usage: "substring(text, start, [end])",
            args: &SUBSTRING_ARGS,
            returns: "Text",
            examples: &SUBSTRING_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &SUBSTRING_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("substring", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let start_idx = match extract_int_or(&args[1..2], 0, 0, "substring", "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let start = normalize_index(start_idx, len);

        let end = if args.len() > 2 {
            match extract_int_or(&args[2..3], 0, len as i64, "substring", "end") {
                Ok(n) => normalize_index(n, len),
                Err(e) => return Value::Error(e),
            }
        } else {
            len
        };

        if start >= len || start >= end {
            return Value::Text(String::new());
        }

        let result: String = chars[start..end.min(len)].iter().collect();
        Value::Text(result)
    }
}

// ============ Left ============

pub struct Left;

static LEFT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of characters",
        optional: false,
        default: None,
    },
];

static LEFT_EXAMPLES: [&str; 1] = [
    "left(\"hello\", 2) → \"he\"",
];

static LEFT_RELATED: [&str; 2] = ["right", "substring"];

impl FunctionPlugin for Left {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "left",
            description: "First n characters",
            usage: "left(text, count)",
            args: &LEFT_ARGS,
            returns: "Text",
            examples: &LEFT_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &LEFT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("left", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let count = match extract_int_or(&args[1..2], 0, 0, "left", "count") {
            Ok(n) if n < 0 => return Value::Error(FolioError::domain_error(
                "left() count must be non-negative"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let result: String = text.chars().take(count).collect();
        Value::Text(result)
    }
}

// ============ Right ============

pub struct Right;

static RIGHT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of characters",
        optional: false,
        default: None,
    },
];

static RIGHT_EXAMPLES: [&str; 1] = [
    "right(\"hello\", 2) → \"lo\"",
];

static RIGHT_RELATED: [&str; 2] = ["left", "substring"];

impl FunctionPlugin for Right {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "right",
            description: "Last n characters",
            usage: "right(text, count)",
            args: &RIGHT_ARGS,
            returns: "Text",
            examples: &RIGHT_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &RIGHT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("right", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let count = match extract_int_or(&args[1..2], 0, 0, "right", "count") {
            Ok(n) if n < 0 => return Value::Error(FolioError::domain_error(
                "right() count must be non-negative"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let chars: Vec<char> = text.chars().collect();
        let start = chars.len().saturating_sub(count);
        let result: String = chars[start..].iter().collect();
        Value::Text(result)
    }
}

// ============ Mid ============

pub struct Mid;

static MID_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Source text",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start position (0-indexed)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of characters",
        optional: false,
        default: None,
    },
];

static MID_EXAMPLES: [&str; 1] = [
    "mid(\"hello\", 1, 3) → \"ell\"",
];

static MID_RELATED: [&str; 2] = ["left", "right"];

impl FunctionPlugin for Mid {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mid",
            description: "Extract count characters starting at index",
            usage: "mid(text, start, count)",
            args: &MID_ARGS,
            returns: "Text",
            examples: &MID_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &MID_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("mid", 3, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_int_or(&args[1..2], 0, 0, "mid", "start") {
            Ok(n) if n < 0 => return Value::Error(FolioError::domain_error(
                "mid() start must be non-negative"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let count = match extract_int_or(&args[2..3], 0, 0, "mid", "count") {
            Ok(n) if n < 0 => return Value::Error(FolioError::domain_error(
                "mid() count must be non-negative"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let result: String = text.chars().skip(start).take(count).collect();
        Value::Text(result)
    }
}

// ============ Split ============

pub struct Split;

static SPLIT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to split",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "delimiter",
        typ: "Text",
        description: "Delimiter string or regex",
        optional: false,
        default: None,
    },
];

static SPLIT_EXAMPLES: [&str; 2] = [
    "split(\"a,b,c\", \",\") → [\"a\", \"b\", \"c\"]",
    "split(\"a  b  c\", r\"\\s+\") → [\"a\", \"b\", \"c\"]",
];

static SPLIT_RELATED: [&str; 2] = ["split_lines", "join"];

impl FunctionPlugin for Split {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "split",
            description: "Split text by delimiter (string or regex)",
            usage: "split(text, delimiter)",
            args: &SPLIT_ARGS,
            returns: "List<Text>",
            examples: &SPLIT_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &SPLIT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("split", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let delimiter = match require_text(&args[1], "split", "delimiter") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        // Try as regex first if it looks like one (contains special chars)
        let parts: Vec<Value> = if delimiter.chars().any(|c| r"\.*+?^${}[]|()".contains(c)) {
            match get_regex(delimiter) {
                Ok(re) => re.split(text).map(|s| Value::Text(s.to_string())).collect(),
                Err(_) => text.split(delimiter).map(|s| Value::Text(s.to_string())).collect(),
            }
        } else {
            text.split(delimiter).map(|s| Value::Text(s.to_string())).collect()
        };

        Value::List(parts)
    }
}

// ============ SplitLines ============

pub struct SplitLines;

static SPLIT_LINES_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to split",
    optional: false,
    default: None,
}];

static SPLIT_LINES_EXAMPLES: [&str; 1] = [
    "split_lines(\"a\\nb\\nc\") → [\"a\", \"b\", \"c\"]",
];

static SPLIT_LINES_RELATED: [&str; 1] = ["split"];

impl FunctionPlugin for SplitLines {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "split_lines",
            description: "Split by newlines (handles \\n, \\r\\n, \\r)",
            usage: "split_lines(text)",
            args: &SPLIT_LINES_ARGS,
            returns: "List<Text>",
            examples: &SPLIT_LINES_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &SPLIT_LINES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("split_lines", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => {
                let lines: Vec<Value> = s
                    .lines()
                    .map(|l| Value::Text(l.to_string()))
                    .collect();
                Value::List(lines)
            }
            Ok(None) => Value::Null,
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Extract ============

pub struct Extract;

static EXTRACT_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search",
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
        name: "default",
        typ: "Text",
        description: "Default if no match",
        optional: true,
        default: Some("\"\""),
    },
];

static EXTRACT_EXAMPLES: [&str; 3] = [
    "extract(\"price: $42.50\", r\"\\d+\\.\\d+\") → \"42.50\"",
    "extract(\"no number\", r\"\\d+\", \"0\") → \"0\"",
    "extract(\"no number\", r\"\\d+\") → \"\"",
];

static EXTRACT_RELATED: [&str; 2] = ["extract_all", "extract_group"];

impl FunctionPlugin for Extract {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extract",
            description: "Extract first regex match",
            usage: "extract(text, pattern, [default])",
            args: &EXTRACT_ARGS,
            returns: "Text",
            examples: &EXTRACT_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("extract", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "extract", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        match re.find(text) {
            Some(m) => Value::Text(m.as_str().to_string()),
            None => {
                if args.len() > 2 {
                    match extract_text(&args[2]) {
                        Ok(Some(default)) => Value::Text(default.to_string()),
                        Ok(None) => Value::Null,
                        Err(e) => Value::Error(e),
                    }
                } else {
                    Value::Text(String::new())
                }
            }
        }
    }
}

// ============ ExtractGroup ============

pub struct ExtractGroup;

static EXTRACT_GROUP_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pattern",
        typ: "Text",
        description: "Regex pattern with capture groups",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "group",
        typ: "Number",
        description: "Group number (1-indexed)",
        optional: false,
        default: None,
    },
];

static EXTRACT_GROUP_EXAMPLES: [&str; 1] = [
    "extract_group(\"John Smith\", r\"(\\w+) (\\w+)\", 2) → \"Smith\"",
];

static EXTRACT_GROUP_RELATED: [&str; 2] = ["extract", "extract_groups"];

impl FunctionPlugin for ExtractGroup {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extract_group",
            description: "Extract specific capture group from first match",
            usage: "extract_group(text, pattern, group)",
            args: &EXTRACT_GROUP_ARGS,
            returns: "Text",
            examples: &EXTRACT_GROUP_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &EXTRACT_GROUP_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("extract_group", 3, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "extract_group", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let group = match extract_int_or(&args[2..3], 0, 1, "extract_group", "group") {
            Ok(n) if n < 1 => return Value::Error(FolioError::domain_error(
                "extract_group() group must be >= 1"
            )),
            Ok(n) => n as usize,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        match re.captures(text) {
            Some(caps) => {
                match caps.get(group) {
                    Some(m) => Value::Text(m.as_str().to_string()),
                    None => Value::Text(String::new()),
                }
            }
            None => Value::Text(String::new()),
        }
    }
}

// ============ ExtractAll ============

pub struct ExtractAll;

static EXTRACT_ALL_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search",
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

static EXTRACT_ALL_EXAMPLES: [&str; 1] = [
    "extract_all(\"a1b2c3\", r\"\\d\") → [\"1\", \"2\", \"3\"]",
];

static EXTRACT_ALL_RELATED: [&str; 1] = ["extract"];

impl FunctionPlugin for ExtractAll {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extract_all",
            description: "Extract all regex matches as list",
            usage: "extract_all(text, pattern)",
            args: &EXTRACT_ALL_ARGS,
            returns: "List<Text>",
            examples: &EXTRACT_ALL_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &EXTRACT_ALL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("extract_all", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "extract_all", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        let matches: Vec<Value> = re
            .find_iter(text)
            .map(|m| Value::Text(m.as_str().to_string()))
            .collect();

        Value::List(matches)
    }
}

// ============ ExtractGroups ============

pub struct ExtractGroups;

static EXTRACT_GROUPS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to search",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pattern",
        typ: "Text",
        description: "Regex pattern with capture groups",
        optional: false,
        default: None,
    },
];

static EXTRACT_GROUPS_EXAMPLES: [&str; 1] = [
    "extract_groups(\"John Smith\", r\"(\\w+) (\\w+)\") → [\"John\", \"Smith\"]",
];

static EXTRACT_GROUPS_RELATED: [&str; 2] = ["extract_group", "extract_all"];

impl FunctionPlugin for ExtractGroups {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extract_groups",
            description: "Extract all capture groups from first match",
            usage: "extract_groups(text, pattern)",
            args: &EXTRACT_GROUPS_ARGS,
            returns: "List<Text>",
            examples: &EXTRACT_GROUPS_EXAMPLES,
            category: "text/extract",
            source: None,
            related: &EXTRACT_GROUPS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("extract_groups", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match require_text(&args[1], "extract_groups", "pattern") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let re = match get_regex(pattern) {
            Ok(r) => r,
            Err(e) => return Value::Error(e),
        };

        match re.captures(text) {
            Some(caps) => {
                let groups: Vec<Value> = caps
                    .iter()
                    .skip(1) // Skip group 0 (full match)
                    .map(|m| {
                        m.map(|m| Value::Text(m.as_str().to_string()))
                            .unwrap_or(Value::Text(String::new()))
                    })
                    .collect();
                Value::List(groups)
            }
            None => Value::List(vec![]),
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
    fn test_len() {
        let f = Len;
        let args = vec![Value::Text("hello".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(5));
    }

    #[test]
    fn test_len_unicode() {
        let f = Len;
        let args = vec![Value::Text("café".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_byte_len() {
        let f = ByteLen;
        let args = vec![Value::Text("café".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(5));
    }

    #[test]
    fn test_substring() {
        let f = Substring;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(4)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "ell");
    }

    #[test]
    fn test_substring_negative() {
        let f = Substring;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Number(Number::from_i64(-3)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "llo");
    }

    #[test]
    fn test_left() {
        let f = Left;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Number(Number::from_i64(2)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "he");
    }

    #[test]
    fn test_right() {
        let f = Right;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Number(Number::from_i64(2)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "lo");
    }

    #[test]
    fn test_split() {
        let f = Split;
        let args = vec![
            Value::Text("a,b,c".to_string()),
            Value::Text(",".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_text().unwrap(), "a");
        assert_eq!(list[1].as_text().unwrap(), "b");
        assert_eq!(list[2].as_text().unwrap(), "c");
    }

    #[test]
    fn test_extract_all() {
        let f = ExtractAll;
        let args = vec![
            Value::Text("a1b2c3".to_string()),
            Value::Text(r"\d".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_text().unwrap(), "1");
    }
}
