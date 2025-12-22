//! Core math functions

use folio_plugin::prelude::*;

pub struct Sqrt;
pub struct Ln;
pub struct Exp;
pub struct Pow;
pub struct Abs;
pub struct Round;
pub struct Floor;
pub struct Ceil;

static SQRT_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value (must be non-negative)", optional: false, default: None }];
static SQRT_EXAMPLES: [&str; 2] = ["sqrt(2)", "sqrt(5)"];
static SQRT_RELATED: [&str; 2] = ["pow", "exp"];

static LN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value (must be positive)", optional: false, default: None }];
static LN_EXAMPLES: [&str; 2] = ["ln(e)", "ln(2)"];
static LN_RELATED: [&str; 1] = ["exp"];

static EXP_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Exponent", optional: false, default: None }];
static EXP_EXAMPLES: [&str; 2] = ["exp(1)", "exp(0)"];
static EXP_RELATED: [&str; 2] = ["ln", "pow"];

static POW_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "base", typ: "Number", description: "Base value", optional: false, default: None },
    ArgMeta { name: "exponent", typ: "Number", description: "Integer exponent", optional: false, default: None },
];
static POW_EXAMPLES: [&str; 2] = ["pow(2, 10)", "pow(phi, 43)"];
static POW_RELATED: [&str; 2] = ["sqrt", "exp"];

static ABS_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value", optional: false, default: None }];
static ABS_EXAMPLES: [&str; 2] = ["abs(-5)", "abs(3.14)"];
static ABS_RELATED: [&str; 0] = [];

static ROUND_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value to round", optional: false, default: None }];
static ROUND_EXAMPLES: [&str; 2] = ["round(3.5)", "round(3.4)"];
static ROUND_RELATED: [&str; 2] = ["floor", "ceil"];

static FLOOR_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value to floor", optional: false, default: None }];
static FLOOR_EXAMPLES: [&str; 2] = ["floor(3.7)", "floor(-2.3)"];
static FLOOR_RELATED: [&str; 2] = ["ceil", "round"];

static CEIL_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value to ceil", optional: false, default: None }];
static CEIL_EXAMPLES: [&str; 2] = ["ceil(3.2)", "ceil(-2.7)"];
static CEIL_RELATED: [&str; 2] = ["floor", "round"];

impl FunctionPlugin for Sqrt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sqrt",
            description: "Square root with arbitrary precision",
            usage: "sqrt(x)",
            args: &SQRT_ARGS,
            returns: "Number",
            examples: &SQRT_EXAMPLES,
            category: "math",
            source: None,
            related: &SQRT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("sqrt", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                match n.sqrt(ctx.precision) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("sqrt", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Ln {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ln",
            description: "Natural logarithm",
            usage: "ln(x)",
            args: &LN_ARGS,
            returns: "Number",
            examples: &LN_EXAMPLES,
            category: "math",
            source: None,
            related: &LN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ln", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                match n.ln(ctx.precision) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("ln", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Exp {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "exp",
            description: "Exponential function (e^x)",
            usage: "exp(x)",
            args: &EXP_ARGS,
            returns: "Number",
            examples: &EXP_EXAMPLES,
            category: "math",
            source: None,
            related: &EXP_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("exp", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.exp(ctx.precision)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("exp", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Pow {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pow",
            description: "Raise to power",
            usage: "pow(base, exponent)",
            args: &POW_ARGS,
            returns: "Number",
            examples: &POW_EXAMPLES,
            category: "math",
            source: None,
            related: &POW_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("pow", 2, args.len()));
        }
        let base = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("pow", "base", "Number", other.type_name())),
        };
        let exp = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("pow", "exponent", "Number", other.type_name())),
        };

        // Use pow_real which handles both integer and fractional exponents
        Value::Number(base.pow_real(exp, ctx.precision))
    }
}

impl FunctionPlugin for Abs {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "abs",
            description: "Absolute value",
            usage: "abs(x)",
            args: &ABS_ARGS,
            returns: "Number",
            examples: &ABS_EXAMPLES,
            category: "math",
            source: None,
            related: &ABS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("abs", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.abs()),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("abs", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Round {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "round",
            description: "Round to nearest integer",
            usage: "round(x)",
            args: &ROUND_ARGS,
            returns: "Number",
            examples: &ROUND_EXAMPLES,
            category: "math",
            source: None,
            related: &ROUND_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("round", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                // Simple rounding via string conversion
                let s = n.as_decimal(0);
                match Number::from_str(&s) {
                    Ok(rounded) => Value::Number(rounded),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("round", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Floor {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "floor",
            description: "Largest integer less than or equal to x",
            usage: "floor(x)",
            args: &FLOOR_ARGS,
            returns: "Number",
            examples: &FLOOR_EXAMPLES,
            category: "math",
            source: None,
            related: &FLOOR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("floor", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.floor()),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("floor", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Ceil {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ceil",
            description: "Smallest integer greater than or equal to x",
            usage: "ceil(x)",
            args: &CEIL_ARGS,
            returns: "Number",
            examples: &CEIL_EXAMPLES,
            category: "math",
            source: None,
            related: &CEIL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ceil", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.ceil()),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("ceil", "x", "Number", other.type_name())),
        }
    }
}
