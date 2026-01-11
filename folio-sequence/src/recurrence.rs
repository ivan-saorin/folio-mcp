//! Custom recurrence relations
//!
//! recurrence, recurrence_named

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_list, require_count};
use crate::expr::eval_recurrence_expr;

// ============ Recurrence ============

pub struct Recurrence;

static RECURRENCE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "initial",
        typ: "List<Number>",
        description: "Initial values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "expr",
        typ: "Text",
        description: "Recurrence expression using a, b, c, d (previous values) and n (index)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Total number of elements to generate",
        optional: false,
        default: None,
    },
];

static RECURRENCE_EXAMPLES: [&str; 4] = [
    "recurrence([1, 1], \"a + b\", 10) → [1, 1, 2, 3, 5, 8, ...]",
    "recurrence([0, 0, 1], \"a + b + c\", 10) → [0, 0, 1, 1, 2, 4, ...]",
    "recurrence([1], \"2 * a\", 8) → [1, 2, 4, 8, 16, ...]",
    "recurrence([1], \"a * n\", 6) → [1, 2, 6, 24, 120, 720]",
];

static RECURRENCE_RELATED: [&str; 2] = ["recurrence_named", "fibonacci"];

impl FunctionPlugin for Recurrence {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "recurrence",
            description: "Generate sequence from custom recurrence relation",
            usage: "recurrence(initial, expr, count)",
            args: &RECURRENCE_ARGS,
            returns: "List<Number>",
            examples: &RECURRENCE_EXAMPLES,
            category: "sequence/recurrence",
            source: None,
            related: &RECURRENCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("recurrence", 3, args.len()));
        }

        let initial = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if initial.is_empty() {
            return Value::Error(FolioError::domain_error(
                "recurrence() requires at least one initial value"
            ));
        }

        let expr_str = match &args[1] {
            Value::Text(s) => s.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("recurrence", "expr", "Text", other.type_name())),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "recurrence", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // Generate sequence
        let mut sequence = initial.clone();

        while sequence.len() < count {
            let n = sequence.len() as i64 + 1; // 1-based index for the new element
            match eval_recurrence_expr(&expr_str, &sequence, n, ctx.precision) {
                Ok(next) => sequence.push(next),
                Err(e) => return Value::Error(e),
            }
        }

        // Truncate to exact count if initial was larger
        sequence.truncate(count);

        let result: Vec<Value> = sequence.into_iter().map(Value::Number).collect();
        Value::List(result)
    }
}

// ============ RecurrenceNamed ============

pub struct RecurrenceNamed;

static RECURRENCE_NAMED_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "name",
        typ: "Text",
        description: "Name of predefined recurrence pattern",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of elements to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index",
        optional: true,
        default: Some("0"),
    },
];

static RECURRENCE_NAMED_EXAMPLES: [&str; 3] = [
    "recurrence_named(\"fibonacci\", 10) → [1, 1, 2, 3, 5, ...]",
    "recurrence_named(\"pell\", 8) → [0, 1, 2, 5, 12, ...]",
    "recurrence_named(\"jacobsthal\", 8) → [0, 1, 1, 3, 5, ...]",
];

static RECURRENCE_NAMED_RELATED: [&str; 2] = ["recurrence", "fibonacci"];

impl FunctionPlugin for RecurrenceNamed {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "recurrence_named",
            description: "Generate sequence from named recurrence pattern",
            usage: "recurrence_named(name, count, [start])",
            args: &RECURRENCE_NAMED_ARGS,
            returns: "List<Number>",
            examples: &RECURRENCE_NAMED_EXAMPLES,
            category: "sequence/recurrence",
            source: None,
            related: &RECURRENCE_NAMED_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 3 {
            return Value::Error(FolioError::arg_count("recurrence_named", 2, args.len()));
        }

        let name = match &args[0] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("recurrence_named", "name", "Text", other.type_name())),
        };

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "recurrence_named", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match args.get(2) {
            Some(Value::Number(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "recurrence_named() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(0) as usize
            }
            Some(Value::Null) | None => 0,
            Some(Value::Error(e)) => return Value::Error(e.clone()),
            Some(other) => return Value::Error(FolioError::arg_type("recurrence_named", "start", "Number", other.type_name())),
        };

        // Get initial values and expression based on name
        let (initial, expr) = match name.as_str() {
            "fibonacci" => (
                vec![Number::from_i64(0), Number::from_i64(1)],
                "a + b",
            ),
            "lucas" => (
                vec![Number::from_i64(2), Number::from_i64(1)],
                "a + b",
            ),
            "pell" => (
                vec![Number::from_i64(0), Number::from_i64(1)],
                "2*a + b",
            ),
            "jacobsthal" => (
                vec![Number::from_i64(0), Number::from_i64(1)],
                "a + 2*b",
            ),
            "tribonacci" => (
                vec![Number::from_i64(0), Number::from_i64(0), Number::from_i64(1)],
                "a + b + c",
            ),
            "padovan" => (
                vec![Number::from_i64(1), Number::from_i64(1), Number::from_i64(1)],
                "b + c",
            ),
            "perrin" => (
                vec![Number::from_i64(3), Number::from_i64(0), Number::from_i64(2)],
                "b + c",
            ),
            _ => {
                return Value::Error(FolioError::domain_error(format!(
                    "Unknown recurrence name: {}. Valid: fibonacci, lucas, pell, jacobsthal, tribonacci, padovan, perrin",
                    name
                )));
            }
        };

        // Generate sequence
        let total_needed = start + count;
        let mut sequence = initial;

        while sequence.len() < total_needed {
            let n = sequence.len() as i64 + 1;
            match eval_recurrence_expr(expr, &sequence, n, ctx.precision) {
                Ok(next) => sequence.push(next),
                Err(e) => return Value::Error(e),
            }
        }

        let result: Vec<Value> = sequence[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_recurrence_fibonacci() {
        let rec = Recurrence;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(1)),
            ]),
            Value::Text("a + b".to_string()),
            Value::Number(Number::from_i64(8)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 8);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[5].as_number().unwrap().to_i64(), Some(8));
        assert_eq!(list[6].as_number().unwrap().to_i64(), Some(13));
        assert_eq!(list[7].as_number().unwrap().to_i64(), Some(21));
    }

    #[test]
    fn test_recurrence_doubling() {
        let rec = Recurrence;
        let args = vec![
            Value::List(vec![Value::Number(Number::from_i64(1))]),
            Value::Text("2 * a".to_string()),
            Value::Number(Number::from_i64(5)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(4));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(8));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(16));
    }

    #[test]
    fn test_recurrence_factorial() {
        let rec = Recurrence;
        let args = vec![
            Value::List(vec![Value::Number(Number::from_i64(1))]),
            Value::Text("a * n".to_string()),
            Value::Number(Number::from_i64(6)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 6);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));  // 1
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(2));  // 1*2
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(6));  // 2*3
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(24)); // 6*4
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(120)); // 24*5
        assert_eq!(list[5].as_number().unwrap().to_i64(), Some(720)); // 120*6
    }

    #[test]
    fn test_recurrence_named_pell() {
        let rec = RecurrenceNamed;
        let args = vec![
            Value::Text("pell".to_string()),
            Value::Number(Number::from_i64(6)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 6);
        // Pell: 0, 1, 2, 5, 12, 29
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(0));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(12));
        assert_eq!(list[5].as_number().unwrap().to_i64(), Some(29));
    }
}
