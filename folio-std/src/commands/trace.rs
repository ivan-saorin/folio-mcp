//! TRACE command

use folio_plugin::prelude::*;

pub struct Trace;

static TRACE_ARGS: [ArgMeta; 1] = [ArgMeta { name: "enabled", typ: "Bool", description: "Enable tracing", optional: true, default: Some("true") }];
static TRACE_EXAMPLES: [&str; 2] = ["TRACE()", "TRACE(false)"];

impl CommandPlugin for Trace {
    fn meta(&self) -> CommandMeta {
        CommandMeta {
            name: "TRACE",
            description: "Enable or disable evaluation tracing",
            args: &TRACE_ARGS,
            examples: &TRACE_EXAMPLES,
        }
    }

    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value {
        let enabled = args.get(0)
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        ctx.tracing = enabled;
        Value::Bool(enabled)
    }
}
