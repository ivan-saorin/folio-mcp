//! EXPLAIN command - show how a value was computed

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct Explain;

static EXPLAIN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "cell", typ: "String", description: "Cell name to explain", optional: false, default: None }];
static EXPLAIN_EXAMPLES: [&str; 2] = ["EXPLAIN(result)", "EXPLAIN(error)"];

impl CommandPlugin for Explain {
    fn meta(&self) -> CommandMeta {
        CommandMeta {
            name: "EXPLAIN",
            description: "Show how a value was computed, including dependencies and intermediate steps",
            args: &EXPLAIN_ARGS,
            examples: &EXPLAIN_EXAMPLES,
        }
    }

    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value {
        // Check argument count
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("EXPLAIN", 1, args.len()));
        }

        // Get cell name
        let cell_name = match &args[0] {
            Value::Text(s) => s.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("EXPLAIN", "cell", "String", other.type_name())),
        };

        // Look up the cell value
        let value = ctx.get_var(&cell_name);

        let mut result = HashMap::new();
        result.insert("cell".to_string(), Value::Text(cell_name.clone()));
        result.insert("value".to_string(), value.clone());

        // Find trace steps for this cell
        let trace_steps: Vec<&TraceStep> = ctx.trace.iter()
            .filter(|step| step.cell == cell_name)
            .collect();

        if !trace_steps.is_empty() {
            let step = trace_steps.last().unwrap();
            result.insert("formula".to_string(), Value::Text(step.formula.clone()));
            result.insert("dependencies".to_string(),
                Value::List(step.dependencies.iter().map(|d| Value::Text(d.clone())).collect()));

            // Build dependency chain
            let mut dep_chain = Vec::new();
            for dep_name in &step.dependencies {
                let dep_value = ctx.get_var(dep_name);
                let mut dep_info = HashMap::new();
                dep_info.insert("name".to_string(), Value::Text(dep_name.to_string()));
                dep_info.insert("value".to_string(), dep_value);

                // Find formula for dependency
                if let Some(dep_step) = ctx.trace.iter().find(|s| &s.cell == dep_name) {
                    dep_info.insert("formula".to_string(), Value::Text(dep_step.formula.clone()));
                }

                dep_chain.push(Value::Object(dep_info));
            }
            result.insert("dependency_values".to_string(), Value::List(dep_chain));
        } else {
            // No trace available - provide basic info
            result.insert("note".to_string(),
                Value::Text("Enable tracing with TRACE(true) to see computation details".to_string()));
        }

        // If it's an error, include error details
        if let Value::Error(e) = &value {
            let mut error_info = HashMap::new();
            error_info.insert("code".to_string(), Value::Text(e.code.clone()));
            error_info.insert("message".to_string(), Value::Text(e.message.clone()));
            if let Some(ref suggestion) = e.suggestion {
                error_info.insert("suggestion".to_string(), Value::Text(suggestion.clone()));
            }
            if let Some(ref context) = e.context {
                if let Some(ref formula) = context.formula {
                    error_info.insert("formula".to_string(), Value::Text(formula.clone()));
                }
                if !context.notes.is_empty() {
                    error_info.insert("notes".to_string(),
                        Value::List(context.notes.iter().map(|n| Value::Text(n.clone())).collect()));
                }
            }
            result.insert("error_details".to_string(), Value::Object(error_info));
        }

        Value::Object(result)
    }
}
