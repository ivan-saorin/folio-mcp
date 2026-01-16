//! Text validation functions: is_empty, is_blank, is_numeric, is_integer, is_alpha,
//! is_alphanumeric, is_email, is_url, is_uuid, is_phone, validate

use folio_plugin::prelude::*;
use crate::helpers::{extract_text, get_regex};
use regex::Regex;
use std::sync::OnceLock;

// ============ Compiled regex patterns ============

fn get_email_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}

fn get_url_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(https?|ftp)://[^\s/$.?#].[^\s]*$").unwrap()
    })
}

fn get_uuid_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").unwrap()
    })
}

fn get_phone_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Matches common phone formats: +1-555-123-4567, (555) 123-4567, 555-123-4567, etc.
        Regex::new(r"^[+]?[\d\s\-().]{7,20}$").unwrap()
    })
}

// ============ IsEmpty ============

pub struct IsEmpty;

static IS_EMPTY_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_EMPTY_EXAMPLES: [&str; 4] = [
    "is_empty(\"\") → true",
    "is_empty(\"  \") → true",
    "is_empty(null) → true",
    "is_empty(\"hello\") → false",
];

static IS_EMPTY_RELATED: [&str; 1] = ["is_blank"];

impl FunctionPlugin for IsEmpty {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_empty",
            description: "Check if text is null, empty, or whitespace-only",
            usage: "is_empty(text)",
            args: &IS_EMPTY_ARGS,
            returns: "Bool",
            examples: &IS_EMPTY_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_EMPTY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_empty", 1, 0));
        }

        match extract_text(&args[0]) {
            Ok(Some(s)) => Value::Bool(s.trim().is_empty()),
            Ok(None) => Value::Bool(true), // null is considered empty
            Err(e) => Value::Error(e),
        }
    }
}

// ============ IsBlank ============

pub struct IsBlank;

static IS_BLANK_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_BLANK_EXAMPLES: [&str; 2] = [
    "is_blank(\"\") → true",
    "is_blank(\"hello\") → false",
];

static IS_BLANK_RELATED: [&str; 1] = ["is_empty"];

impl FunctionPlugin for IsBlank {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_blank",
            description: "Check if text is null, empty, or whitespace-only (alias for is_empty)",
            usage: "is_blank(text)",
            args: &IS_BLANK_ARGS,
            returns: "Bool",
            examples: &IS_BLANK_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_BLANK_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Delegate to is_empty
        IsEmpty.call(args, ctx)
    }
}

// ============ IsNumeric ============

pub struct IsNumeric;

static IS_NUMERIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_NUMERIC_EXAMPLES: [&str; 4] = [
    "is_numeric(\"123\") → true",
    "is_numeric(\"12.34\") → true",
    "is_numeric(\"1e5\") → true",
    "is_numeric(\"abc\") → false",
];

static IS_NUMERIC_RELATED: [&str; 2] = ["is_integer", "parse_number"];

impl FunctionPlugin for IsNumeric {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_numeric",
            description: "Check if text is parseable as a number",
            usage: "is_numeric(text)",
            args: &IS_NUMERIC_ARGS,
            returns: "Bool",
            examples: &IS_NUMERIC_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_NUMERIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_numeric", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        Value::Bool(Number::from_str(text.trim()).is_ok())
    }
}

// ============ IsInteger ============

pub struct IsInteger;

static IS_INTEGER_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_INTEGER_EXAMPLES: [&str; 2] = [
    "is_integer(\"123\") → true",
    "is_integer(\"12.34\") → false",
];

static IS_INTEGER_RELATED: [&str; 2] = ["is_numeric", "parse_int"];

impl FunctionPlugin for IsInteger {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_integer",
            description: "Check if text is parseable as an integer",
            usage: "is_integer(text)",
            args: &IS_INTEGER_ARGS,
            returns: "Bool",
            examples: &IS_INTEGER_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_INTEGER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_integer", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        let trimmed = text.trim();

        // Check if it contains a decimal point or scientific notation with decimal
        if trimmed.contains('.') {
            return Value::Bool(false);
        }

        // Check if parseable as integer
        Value::Bool(trimmed.parse::<i64>().is_ok())
    }
}

// ============ IsAlpha ============

pub struct IsAlpha;

static IS_ALPHA_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_ALPHA_EXAMPLES: [&str; 3] = [
    "is_alpha(\"hello\") → true",
    "is_alpha(\"hello1\") → false",
    "is_alpha(\"café\") → true",
];

static IS_ALPHA_RELATED: [&str; 1] = ["is_alphanumeric"];

impl FunctionPlugin for IsAlpha {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_alpha",
            description: "Check if text contains only letters (Unicode aware)",
            usage: "is_alpha(text)",
            args: &IS_ALPHA_ARGS,
            returns: "Bool",
            examples: &IS_ALPHA_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_ALPHA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_alpha", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        if text.is_empty() {
            return Value::Bool(false);
        }

        Value::Bool(text.chars().all(|c| c.is_alphabetic()))
    }
}

// ============ IsAlphanumeric ============

pub struct IsAlphanumeric;

static IS_ALPHANUMERIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_ALPHANUMERIC_EXAMPLES: [&str; 2] = [
    "is_alphanumeric(\"hello123\") → true",
    "is_alphanumeric(\"hello!\") → false",
];

static IS_ALPHANUMERIC_RELATED: [&str; 1] = ["is_alpha"];

impl FunctionPlugin for IsAlphanumeric {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_alphanumeric",
            description: "Check if text contains only letters and digits",
            usage: "is_alphanumeric(text)",
            args: &IS_ALPHANUMERIC_ARGS,
            returns: "Bool",
            examples: &IS_ALPHANUMERIC_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_ALPHANUMERIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_alphanumeric", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        if text.is_empty() {
            return Value::Bool(false);
        }

        Value::Bool(text.chars().all(|c| c.is_alphanumeric()))
    }
}

// ============ IsEmail ============

pub struct IsEmail;

static IS_EMAIL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_EMAIL_EXAMPLES: [&str; 2] = [
    "is_email(\"user@example.com\") → true",
    "is_email(\"invalid\") → false",
];

static IS_EMAIL_RELATED: [&str; 1] = ["is_url"];

impl FunctionPlugin for IsEmail {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_email",
            description: "Check if text is a valid email format (simplified RFC 5322)",
            usage: "is_email(text)",
            args: &IS_EMAIL_ARGS,
            returns: "Bool",
            examples: &IS_EMAIL_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_EMAIL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_email", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        let re = get_email_regex();
        Value::Bool(re.is_match(text.trim()))
    }
}

// ============ IsUrl ============

pub struct IsUrl;

static IS_URL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_URL_EXAMPLES: [&str; 2] = [
    "is_url(\"https://example.com\") → true",
    "is_url(\"not a url\") → false",
];

static IS_URL_RELATED: [&str; 1] = ["is_email"];

impl FunctionPlugin for IsUrl {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_url",
            description: "Check if text is a valid URL format",
            usage: "is_url(text)",
            args: &IS_URL_ARGS,
            returns: "Bool",
            examples: &IS_URL_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_URL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_url", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        let re = get_url_regex();
        Value::Bool(re.is_match(text.trim()))
    }
}

// ============ IsUuid ============

pub struct IsUuid;

static IS_UUID_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "text",
    typ: "Text",
    description: "Text to check",
    optional: false,
    default: None,
}];

static IS_UUID_EXAMPLES: [&str; 1] = [
    "is_uuid(\"550e8400-e29b-41d4-a716-446655440000\") → true",
];

static IS_UUID_RELATED: [&str; 1] = ["validate"];

impl FunctionPlugin for IsUuid {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_uuid",
            description: "Check if text is a valid UUID format",
            usage: "is_uuid(text)",
            args: &IS_UUID_ARGS,
            returns: "Bool",
            examples: &IS_UUID_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_UUID_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_uuid", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        let re = get_uuid_regex();
        Value::Bool(re.is_match(text.trim()))
    }
}

// ============ IsPhone ============

pub struct IsPhone;

static IS_PHONE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to check",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "region",
        typ: "Text",
        description: "Region code (currently unused)",
        optional: true,
        default: None,
    },
];

static IS_PHONE_EXAMPLES: [&str; 1] = [
    "is_phone(\"+1-555-123-4567\") → true",
];

static IS_PHONE_RELATED: [&str; 1] = ["validate"];

impl FunctionPlugin for IsPhone {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_phone",
            description: "Check if text is a valid phone number format",
            usage: "is_phone(text, [region])",
            args: &IS_PHONE_ARGS,
            returns: "Bool",
            examples: &IS_PHONE_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &IS_PHONE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_phone", 1, 0));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Bool(false),
            Err(e) => return Value::Error(e),
        };

        // Region is currently unused but accepted for future expansion
        let _region = match args.get(1) {
            Some(Value::Text(s)) => Some(s.as_str()),
            Some(Value::Null) | None => None,
            Some(Value::Error(e)) => return Value::Error(e.clone()),
            _ => None,
        };

        let re = get_phone_regex();
        let trimmed = text.trim();

        // Additional check: must contain at least some digits
        let digit_count = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count < 7 {
            return Value::Bool(false);
        }

        Value::Bool(re.is_match(trimmed))
    }
}

// ============ Validate ============

pub struct Validate;

static VALIDATE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Text to validate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pattern",
        typ: "Text",
        description: "Regex pattern to match against",
        optional: false,
        default: None,
    },
];

static VALIDATE_EXAMPLES: [&str; 1] = [
    "validate(\"AB123\", r\"^[A-Z]{2}\\d{3}$\") → true",
];

static VALIDATE_RELATED: [&str; 2] = ["matches", "extract"];

impl FunctionPlugin for Validate {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "validate",
            description: "Validate text against a regex pattern",
            usage: "validate(text, pattern)",
            args: &VALIDATE_ARGS,
            returns: "Bool",
            examples: &VALIDATE_EXAMPLES,
            category: "text/validate",
            source: None,
            related: &VALIDATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("validate", 2, args.len()));
        }

        let text = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let pattern = match extract_text(&args[1]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Error(FolioError::arg_type(
                "validate", "pattern", "Text", "Null"
            )),
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
    fn test_is_empty() {
        let f = IsEmpty;

        assert_eq!(
            f.call(&[Value::Text("".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("  ".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Null], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("hello".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_numeric() {
        let f = IsNumeric;

        assert_eq!(
            f.call(&[Value::Text("123".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("12.34".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("1e5".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("abc".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_integer() {
        let f = IsInteger;

        assert_eq!(
            f.call(&[Value::Text("123".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("-456".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("12.34".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_alpha() {
        let f = IsAlpha;

        assert_eq!(
            f.call(&[Value::Text("hello".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("café".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("hello1".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_alphanumeric() {
        let f = IsAlphanumeric;

        assert_eq!(
            f.call(&[Value::Text("hello123".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("hello!".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_email() {
        let f = IsEmail;

        assert_eq!(
            f.call(&[Value::Text("user@example.com".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("invalid".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
        assert_eq!(
            f.call(&[Value::Text("user@domain".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_url() {
        let f = IsUrl;

        assert_eq!(
            f.call(&[Value::Text("https://example.com".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("http://example.com/path?query=1".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("not a url".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_uuid() {
        let f = IsUuid;

        assert_eq!(
            f.call(&[Value::Text("550e8400-e29b-41d4-a716-446655440000".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("not-a-uuid".to_string())], &eval_ctx()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_is_phone() {
        let f = IsPhone;

        assert_eq!(
            f.call(&[Value::Text("+1-555-123-4567".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("(555) 123-4567".to_string())], &eval_ctx()).as_bool(),
            Some(true)
        );
        assert_eq!(
            f.call(&[Value::Text("123".to_string())], &eval_ctx()).as_bool(),
            Some(false) // Too short
        );
    }

    #[test]
    fn test_validate() {
        let f = Validate;

        let args = vec![
            Value::Text("AB123".to_string()),
            Value::Text(r"^[A-Z]{2}\d{3}$".to_string()),
        ];
        assert_eq!(f.call(&args, &eval_ctx()).as_bool(), Some(true));

        let args2 = vec![
            Value::Text("abc".to_string()),
            Value::Text(r"^[A-Z]{2}\d{3}$".to_string()),
        ];
        assert_eq!(f.call(&args2, &eval_ctx()).as_bool(), Some(false));
    }
}
