//! Aggregate functions

use folio_plugin::prelude::*;

pub struct Sum;

static SUM_ARGS: [ArgMeta; 1] = [ArgMeta { name: "values", typ: "Number...", description: "Values to sum", optional: false, default: None }];
static SUM_EXAMPLES: [&str; 1] = ["sum(1, 2, 3)"];
static SUM_RELATED: [&str; 0] = [];

impl FunctionPlugin for Sum {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sum",
            description: "Sum of values",
            usage: "sum(a, b, ...)",
            args: &SUM_ARGS,
            returns: "Number",
            examples: &SUM_EXAMPLES,
            category: "aggregate",
            source: None,
            related: &SUM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let mut total = Number::from_i64(0);
        for arg in args {
            match arg {
                Value::Number(n) => total = total.add(n),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("sum", "values", "Number", other.type_name())),
            }
        }
        Value::Number(total)
    }
}
