//! Trigonometric functions

use folio_plugin::prelude::*;

pub struct Sin;
pub struct Cos;
pub struct Tan;

static SIN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Angle in radians", optional: false, default: None }];
static SIN_EXAMPLES: [&str; 2] = ["sin(0)", "sin(π/2)"];
static SIN_RELATED: [&str; 2] = ["cos", "tan"];

static COS_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Angle in radians", optional: false, default: None }];
static COS_EXAMPLES: [&str; 2] = ["cos(0)", "cos(π)"];
static COS_RELATED: [&str; 2] = ["sin", "tan"];

static TAN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Angle in radians", optional: false, default: None }];
static TAN_EXAMPLES: [&str; 2] = ["tan(0)", "tan(π/4)"];
static TAN_RELATED: [&str; 2] = ["sin", "cos"];

impl FunctionPlugin for Sin {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sin",
            description: "Sine function",
            usage: "sin(x)",
            args: &SIN_ARGS,
            returns: "Number",
            examples: &SIN_EXAMPLES,
            category: "trig",
            source: None,
            related: &SIN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("sin", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.sin(ctx.precision)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("sin", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Cos {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cos",
            description: "Cosine function",
            usage: "cos(x)",
            args: &COS_ARGS,
            returns: "Number",
            examples: &COS_EXAMPLES,
            category: "trig",
            source: None,
            related: &COS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("cos", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.cos(ctx.precision)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("cos", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Tan {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tan",
            description: "Tangent function",
            usage: "tan(x)",
            args: &TAN_ARGS,
            returns: "Number",
            examples: &TAN_EXAMPLES,
            category: "trig",
            source: None,
            related: &TAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("tan", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                match n.tan(ctx.precision) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("tan", "x", "Number", other.type_name())),
        }
    }
}
