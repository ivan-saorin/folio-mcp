//! Confidence interval functions: ci, moe

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_min_count, mean, variance_impl};
use crate::distributions::normal::standard_normal_inv;
use std::collections::HashMap;

// ============ Confidence Interval ============

pub struct Ci;

static CI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sample data",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "level",
        typ: "Number",
        description: "Confidence level (0-1, default 0.95)",
        optional: true,
        default: Some("0.95"),
    },
];

static CI_EXAMPLES: [&str; 2] = [
    "ci([1,2,3,4,5]) → {low: ..., high: ...}",
    "ci([1,2,3,4,5], 0.99)",
];

static CI_RELATED: [&str; 2] = ["moe", "se"];

impl FunctionPlugin for Ci {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ci",
            description: "Confidence interval for the mean",
            usage: "ci(list, level?)",
            args: &CI_ARGS,
            returns: "Object",
            examples: &CI_EXAMPLES,
            category: "stats/confidence",
            source: None,
            related: &CI_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("ci", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let level = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.95),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("ci", "level", "Number", other.type_name())),
            }
        } else {
            0.95
        };

        if level <= 0.0 || level >= 1.0 {
            return Value::Error(FolioError::domain_error(
                "ci() requires 0 < level < 1",
            ));
        }

        if let Err(e) = require_min_count(&numbers, 2, "ci") {
            return Value::Error(e);
        }

        let n = numbers.len();
        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let se = match sd.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // z-score for given confidence level
        let alpha = (1.0 - level) / 2.0;
        let p = Number::from_str(&format!("{:.15}", 1.0 - alpha)).unwrap_or(Number::from_str("0.975").unwrap());
        let z = standard_normal_inv(&p, ctx.precision);

        let margin = se.mul(&z);
        let ci_low = m.sub(&margin);
        let ci_high = m.add(&margin);

        let mut result = HashMap::new();
        result.insert("low".to_string(), Value::Number(ci_low));
        result.insert("high".to_string(), Value::Number(ci_high));
        result.insert("margin".to_string(), Value::Number(margin));
        result.insert("level".to_string(), Value::Number(Number::from_str(&format!("{:.15}", level)).unwrap_or(Number::from_str("0.95").unwrap())));

        Value::Object(result)
    }
}

// ============ Margin of Error ============

pub struct Moe;

static MOE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sample data",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "level",
        typ: "Number",
        description: "Confidence level (0-1, default 0.95)",
        optional: true,
        default: Some("0.95"),
    },
];

static MOE_EXAMPLES: [&str; 1] = ["moe([1,2,3,4,5]) → margin value"];

static MOE_RELATED: [&str; 2] = ["ci", "se"];

impl FunctionPlugin for Moe {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "moe",
            description: "Margin of error",
            usage: "moe(list, level?)",
            args: &MOE_ARGS,
            returns: "Number",
            examples: &MOE_EXAMPLES,
            category: "stats/confidence",
            source: None,
            related: &MOE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("moe", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let level = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.95),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("moe", "level", "Number", other.type_name())),
            }
        } else {
            0.95
        };

        if level <= 0.0 || level >= 1.0 {
            return Value::Error(FolioError::domain_error(
                "moe() requires 0 < level < 1",
            ));
        }

        if let Err(e) = require_min_count(&numbers, 2, "moe") {
            return Value::Error(e);
        }

        let n = numbers.len();

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let se = match sd.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // z-score for given confidence level
        let alpha = (1.0 - level) / 2.0;
        let p = Number::from_str(&format!("{:.15}", 1.0 - alpha)).unwrap_or(Number::from_str("0.975").unwrap());
        let z = standard_normal_inv(&p, ctx.precision);

        Value::Number(se.mul(&z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_ci() {
        let ci = Ci;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = ci.call(&args, &eval_ctx());
        assert!(result.as_object().is_some());
    }

    #[test]
    fn test_moe() {
        let moe = Moe;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = moe.call(&args, &eval_ctx());
        assert!(result.as_number().is_some());
    }
}
