//! Text join functions: concat, join, format, template

use folio_plugin::prelude::*;
use crate::helpers::{extract_text, require_text};

// ============ Concat ============

pub struct Concat;

static CONCAT_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "texts",
    typ: "Text...",
    description: "Texts to concatenate",
    optional: false,
    default: None,
}];

static CONCAT_EXAMPLES: [&str; 1] = [
    "concat(\"hello\", \" \", \"world\") → \"hello world\"",
];

static CONCAT_RELATED: [&str; 1] = ["join"];

impl FunctionPlugin for Concat {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "concat",
            description: "Concatenate multiple texts",
            usage: "concat(text1, text2, ...)",
            args: &CONCAT_ARGS,
            returns: "Text",
            examples: &CONCAT_EXAMPLES,
            category: "text/join",
            source: None,
            related: &CONCAT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let mut result = String::new();

        for arg in args {
            match arg {
                Value::Text(s) => result.push_str(s),
                Value::Number(n) => result.push_str(&n.as_decimal(15)),
                Value::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
                Value::Null => {} // Skip nulls
                Value::Error(e) => return Value::Error(e.clone()),
                Value::List(list) => {
                    for item in list {
                        match item {
                            Value::Text(s) => result.push_str(s),
                            Value::Number(n) => result.push_str(&n.as_decimal(15)),
                            Value::Error(e) => return Value::Error(e.clone()),
                            _ => {}
                        }
                    }
                }
                _ => {} // Skip other types
            }
        }

        Value::Text(result)
    }
}

// ============ Join ============

pub struct Join;

static JOIN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Text>",
        description: "List of texts to join",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "delimiter",
        typ: "Text",
        description: "Delimiter to insert between elements",
        optional: false,
        default: None,
    },
];

static JOIN_EXAMPLES: [&str; 1] = [
    "join([\"a\", \"b\", \"c\"], \", \") → \"a, b, c\"",
];

static JOIN_RELATED: [&str; 2] = ["concat", "split"];

impl FunctionPlugin for Join {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "join",
            description: "Join list elements with delimiter",
            usage: "join(list, delimiter)",
            args: &JOIN_ARGS,
            returns: "Text",
            examples: &JOIN_EXAMPLES,
            category: "text/join",
            source: None,
            related: &JOIN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("join", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Null => return Value::Null,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "join", "list", "List", other.type_name()
            )),
        };

        let delimiter = match require_text(&args[1], "join", "delimiter") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        let parts: Vec<String> = list.iter()
            .filter_map(|v| match v {
                Value::Text(s) => Some(s.clone()),
                Value::Number(n) => Some(n.as_decimal(15)),
                Value::Bool(b) => Some(if *b { "true".to_string() } else { "false".to_string() }),
                _ => None,
            })
            .collect();

        Value::Text(parts.join(delimiter))
    }
}

// ============ Format ============

pub struct Format;

static FORMAT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "template",
        typ: "Text",
        description: "Template with {0}, {1} or {name} placeholders",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "values",
        typ: "List | Object",
        description: "Values to substitute",
        optional: false,
        default: None,
    },
];

static FORMAT_EXAMPLES: [&str; 2] = [
    "format(\"{0} is {1}\", [\"answer\", 42]) → \"answer is 42\"",
    "format(\"{name}: {value}\", {name: \"x\", value: 10}) → \"x: 10\"",
];

static FORMAT_RELATED: [&str; 1] = ["template"];

impl FunctionPlugin for Format {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "format",
            description: "Format string with positional {0} or named {name} placeholders",
            usage: "format(template, values)",
            args: &FORMAT_ARGS,
            returns: "Text",
            examples: &FORMAT_EXAMPLES,
            category: "text/join",
            source: None,
            related: &FORMAT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("format", 2, args.len()));
        }

        let template = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let mut result = template.to_string();

        match &args[1] {
            Value::List(list) => {
                // Positional substitution: {0}, {1}, etc.
                for (i, val) in list.iter().enumerate() {
                    let placeholder = format!("{{{}}}", i);
                    let replacement = value_to_string(val);
                    result = result.replace(&placeholder, &replacement);
                }
            }
            Value::Object(obj) => {
                // Named substitution: {name}, {value}, etc.
                for (key, val) in obj {
                    let placeholder = format!("{{{}}}", key);
                    let replacement = value_to_string(val);
                    result = result.replace(&placeholder, &replacement);
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "format", "values", "List or Object", other.type_name()
            )),
        }

        Value::Text(result)
    }
}

// ============ Template ============

pub struct Template;

static TEMPLATE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "text",
        typ: "Text",
        description: "Template with {{name}} placeholders",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "vars",
        typ: "Object",
        description: "Variables to substitute",
        optional: false,
        default: None,
    },
];

static TEMPLATE_EXAMPLES: [&str; 1] = [
    "template(\"Hello {{name}}!\", {name: \"World\"}) → \"Hello World!\"",
];

static TEMPLATE_RELATED: [&str; 1] = ["format"];

impl FunctionPlugin for Template {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "template",
            description: "Mustache-style template substitution with {{name}}",
            usage: "template(text, vars)",
            args: &TEMPLATE_ARGS,
            returns: "Text",
            examples: &TEMPLATE_EXAMPLES,
            category: "text/join",
            source: None,
            related: &TEMPLATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("template", 2, args.len()));
        }

        let template = match extract_text(&args[0]) {
            Ok(Some(s)) => s,
            Ok(None) => return Value::Null,
            Err(e) => return Value::Error(e),
        };

        let vars = match &args[1] {
            Value::Object(obj) => obj,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type(
                "template", "vars", "Object", other.type_name()
            )),
        };

        let mut result = template.to_string();

        for (key, val) in vars {
            let placeholder = format!("{{{{{}}}}}", key); // {{key}}
            let replacement = value_to_string(val);
            result = result.replace(&placeholder, &replacement);
        }

        Value::Text(result)
    }
}

/// Convert a Value to its string representation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Text(s) => s.clone(),
        Value::Number(n) => n.as_decimal(15).trim_end_matches('0').trim_end_matches('.').to_string(),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::Null => String::new(),
        Value::DateTime(dt) => dt.to_iso_string(),
        Value::Duration(d) => format!("{} seconds", d.as_secs()),
        Value::List(_) => "[List]".to_string(),
        Value::Object(_) => "[Object]".to_string(),
        Value::Error(e) => format!("[Error: {}]", e.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_concat() {
        let f = Concat;
        let args = vec![
            Value::Text("hello".to_string()),
            Value::Text(" ".to_string()),
            Value::Text("world".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "hello world");
    }

    #[test]
    fn test_join() {
        let f = Join;
        let args = vec![
            Value::List(vec![
                Value::Text("a".to_string()),
                Value::Text("b".to_string()),
                Value::Text("c".to_string()),
            ]),
            Value::Text(", ".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "a, b, c");
    }

    #[test]
    fn test_format_positional() {
        let f = Format;
        let args = vec![
            Value::Text("{0} is {1}".to_string()),
            Value::List(vec![
                Value::Text("answer".to_string()),
                Value::Number(Number::from_i64(42)),
            ]),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "answer is 42");
    }

    #[test]
    fn test_format_named() {
        let f = Format;
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Value::Text("x".to_string()));
        obj.insert("value".to_string(), Value::Number(Number::from_i64(10)));

        let args = vec![
            Value::Text("{name}: {value}".to_string()),
            Value::Object(obj),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "x: 10");
    }

    #[test]
    fn test_template() {
        let f = Template;
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Value::Text("World".to_string()));

        let args = vec![
            Value::Text("Hello {{name}}!".to_string()),
            Value::Object(obj),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_text().unwrap(), "Hello World!");
    }
}
