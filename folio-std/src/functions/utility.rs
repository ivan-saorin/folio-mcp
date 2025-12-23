//! Utility functions for LLM experience: fields, head, tail, typeof, describe

use folio_plugin::prelude::*;
use std::collections::HashMap;

// ============================================================================
// fields(object) → List<Text>
// ============================================================================

pub struct FieldsFn;

static FIELDS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "object",
    typ: "Object",
    description: "Object to get fields from",
    optional: false,
    default: None,
}];
static FIELDS_EXAMPLES: [&str; 1] = ["fields(linear_reg(x, y)) → [\"slope\", \"intercept\", ...]"];
static FIELDS_RELATED: [&str; 2] = ["describe", "typeof"];

impl FunctionPlugin for FieldsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "fields",
            description: "List available fields on an Object",
            usage: "fields(object)",
            args: &FIELDS_ARGS,
            returns: "List<Text>",
            examples: &FIELDS_EXAMPLES,
            category: "utility",
            source: None,
            related: &FIELDS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("fields", 1, args.len()));
        }

        match &args[0] {
            Value::Object(obj) => {
                let mut keys: Vec<String> = obj.keys().cloned().collect();
                keys.sort();
                Value::List(keys.into_iter().map(Value::Text).collect())
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(
                FolioError::arg_type("fields", "object", "Object", other.type_name())
                    .with_suggestion("fields() shows available fields on Objects returned by functions like linear_reg(), t_test_1(), etc.")
            ),
        }
    }
}

// ============================================================================
// head(list, n?) → List
// ============================================================================

pub struct HeadFn;

static HEAD_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get elements from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements (default: 5)",
        optional: true,
        default: Some("5"),
    },
];
static HEAD_EXAMPLES: [&str; 2] = ["head([1,2,3,4,5,6], 3) → [1, 2, 3]", "head(data) → first 5 elements"];
static HEAD_RELATED: [&str; 2] = ["tail", "take"];

impl FunctionPlugin for HeadFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "head",
            description: "First n elements of list",
            usage: "head(list, n?)",
            args: &HEAD_ARGS,
            returns: "List",
            examples: &HEAD_EXAMPLES,
            category: "utility",
            source: None,
            related: &HEAD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("head", 1, args.len())
                .with_suggestion("Usage: head(list) or head(list, n)"));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("head", "list", "List", other.type_name())),
        };

        let n = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_i64().unwrap_or(5) as usize,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("head", "n", "Number", other.type_name())),
            }
        } else {
            5
        };

        Value::List(list.iter().take(n).cloned().collect())
    }
}

// ============================================================================
// tail(list, n?) → List
// ============================================================================

pub struct TailFn;

static TAIL_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get elements from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements (default: 5)",
        optional: true,
        default: Some("5"),
    },
];
static TAIL_EXAMPLES: [&str; 2] = ["tail([1,2,3,4,5,6], 3) → [4, 5, 6]", "tail(data) → last 5 elements"];
static TAIL_RELATED: [&str; 2] = ["head", "take"];

impl FunctionPlugin for TailFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tail",
            description: "Last n elements of list",
            usage: "tail(list, n?)",
            args: &TAIL_ARGS,
            returns: "List",
            examples: &TAIL_EXAMPLES,
            category: "utility",
            source: None,
            related: &TAIL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("tail", 1, args.len())
                .with_suggestion("Usage: tail(list) or tail(list, n)"));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("tail", "list", "List", other.type_name())),
        };

        let n = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_i64().unwrap_or(5) as usize,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("tail", "n", "Number", other.type_name())),
            }
        } else {
            5
        };

        let skip = list.len().saturating_sub(n);
        Value::List(list.iter().skip(skip).cloned().collect())
    }
}

// ============================================================================
// take(list, n) → List (alias for head)
// ============================================================================

pub struct TakeFn;

static TAKE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get elements from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements",
        optional: false,
        default: None,
    },
];
static TAKE_EXAMPLES: [&str; 1] = ["take([1,2,3,4,5], 3) → [1, 2, 3]"];
static TAKE_RELATED: [&str; 2] = ["head", "tail"];

impl FunctionPlugin for TakeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "take",
            description: "First n elements of list (alias for head)",
            usage: "take(list, n)",
            args: &TAKE_ARGS,
            returns: "List",
            examples: &TAKE_EXAMPLES,
            category: "utility",
            source: None,
            related: &TAKE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Delegate to head
        HeadFn.call(args, ctx)
    }
}

// ============================================================================
// typeof(value) → Text
// ============================================================================

pub struct TypeofFn;

static TYPEOF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "value",
    typ: "Any",
    description: "Value to get type of",
    optional: false,
    default: None,
}];
static TYPEOF_EXAMPLES: [&str; 3] = [
    "typeof(42) → \"Number\"",
    "typeof([1,2,3]) → \"List\"",
    "typeof(linear_reg(x,y)) → \"Object\"",
];
static TYPEOF_RELATED: [&str; 2] = ["fields", "describe"];

impl FunctionPlugin for TypeofFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "typeof",
            description: "Get type name of a value",
            usage: "typeof(value)",
            args: &TYPEOF_ARGS,
            returns: "Text",
            examples: &TYPEOF_EXAMPLES,
            category: "utility",
            source: None,
            related: &TYPEOF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("typeof", 1, args.len()));
        }

        Value::Text(args[0].type_name().to_string())
    }
}

// ============================================================================
// describe(object) → Object
// ============================================================================

pub struct DescribeFn;

static DESCRIBE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "object",
    typ: "Object",
    description: "Object to describe",
    optional: false,
    default: None,
}];
static DESCRIBE_EXAMPLES: [&str; 1] = ["describe(linear_reg(x, y)) → detailed Object info"];
static DESCRIBE_RELATED: [&str; 2] = ["fields", "typeof"];

impl FunctionPlugin for DescribeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "describe",
            description: "Full description of Object with field values and types",
            usage: "describe(object)",
            args: &DESCRIBE_ARGS,
            returns: "Object",
            examples: &DESCRIBE_EXAMPLES,
            category: "utility",
            source: None,
            related: &DESCRIBE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("describe", 1, args.len()));
        }

        match &args[0] {
            Value::Object(obj) => {
                let mut result = HashMap::new();
                result.insert("type".to_string(), Value::Text("Object".to_string()));
                result.insert("field_count".to_string(), Value::Number(Number::from_i64(obj.len() as i64)));

                let mut fields_info = HashMap::new();
                for (key, value) in obj {
                    let mut field = HashMap::new();
                    field.insert("type".to_string(), Value::Text(value.type_name().to_string()));
                    field.insert("value".to_string(), value.clone());
                    fields_info.insert(key.clone(), Value::Object(field));
                }
                result.insert("fields".to_string(), Value::Object(fields_info));

                Value::Object(result)
            }
            Value::List(list) => {
                let mut result = HashMap::new();
                result.insert("type".to_string(), Value::Text("List".to_string()));
                result.insert("length".to_string(), Value::Number(Number::from_i64(list.len() as i64)));

                // Show first 5 elements
                let preview: Vec<Value> = list.iter().take(5).cloned().collect();
                result.insert("preview".to_string(), Value::List(preview));

                // Count by type
                let mut type_counts: HashMap<String, i64> = HashMap::new();
                for item in list {
                    *type_counts.entry(item.type_name().to_string()).or_insert(0) += 1;
                }
                let types_obj: HashMap<String, Value> = type_counts
                    .into_iter()
                    .map(|(k, v)| (k, Value::Number(Number::from_i64(v))))
                    .collect();
                result.insert("element_types".to_string(), Value::Object(types_obj));

                Value::Object(result)
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => {
                let mut result = HashMap::new();
                result.insert("type".to_string(), Value::Text(other.type_name().to_string()));
                result.insert("value".to_string(), other.clone());
                Value::Object(result)
            }
        }
    }
}

// ============================================================================
// len(list) → Number
// ============================================================================

pub struct LenFn;

static LEN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List or Text",
    description: "List or Text to get length of",
    optional: false,
    default: None,
}];
static LEN_EXAMPLES: [&str; 2] = ["len([1,2,3]) → 3", "len(\"hello\") → 5"];
static LEN_RELATED: [&str; 2] = ["count", "head"];

impl FunctionPlugin for LenFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "len",
            description: "Length of a list or text",
            usage: "len(value)",
            args: &LEN_ARGS,
            returns: "Number",
            examples: &LEN_EXAMPLES,
            category: "utility",
            source: None,
            related: &LEN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("len", 1, args.len()));
        }

        match &args[0] {
            Value::List(list) => Value::Number(Number::from_i64(list.len() as i64)),
            Value::Text(s) => Value::Number(Number::from_i64(s.len() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("len", "list", "List or Text", other.type_name())),
        }
    }
}

// ============================================================================
// nth(list, index) → Value
// ============================================================================

pub struct NthFn;

static NTH_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get element from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "index",
        typ: "Number",
        description: "Zero-based index",
        optional: false,
        default: None,
    },
];
static NTH_EXAMPLES: [&str; 2] = ["nth([10, 20, 30], 0) → 10", "nth([10, 20, 30], 2) → 30"];
static NTH_RELATED: [&str; 2] = ["head", "tail"];

impl FunctionPlugin for NthFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nth",
            description: "Get element at index (0-based)",
            usage: "nth(list, index)",
            args: &NTH_ARGS,
            returns: "Value",
            examples: &NTH_EXAMPLES,
            category: "utility",
            source: None,
            related: &NTH_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("nth", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("nth", "list", "List", other.type_name())),
        };

        let index = match &args[1] {
            Value::Number(n) => n.to_i64().unwrap_or(-1),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("nth", "index", "Number", other.type_name())),
        };

        if index < 0 || index as usize >= list.len() {
            return Value::Error(FolioError::domain_error(format!(
                "Index {} out of bounds for list of length {}",
                index, list.len()
            )));
        }

        list[index as usize].clone()
    }
}
