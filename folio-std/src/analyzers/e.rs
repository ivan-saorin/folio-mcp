//! Euler's number (e) pattern detection

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct EAnalyzer;

impl AnalyzerPlugin for EAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "e",
            description: "Detects Euler's number patterns (e, e², ln(n), etc.)",
            detects: &["e", "e²", "1/e", "e^n"],
        }
    }

    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }

    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let e = Number::e(ctx.precision);
        let mut result = HashMap::new();

        // Test e^n for small integer n
        for n in -5i32..=5 {
            if n == 0 {
                continue; // e^0 = 1 is not interesting
            }
            let e_n = e.pow(n);
            if let Ok(ratio) = value.checked_div(&e_n) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 && i != 0 {
                        let key = if n == 1 {
                            "e".to_string()
                        } else {
                            format!("e^{}", n)
                        };
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.85").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Check if value is a natural logarithm of a small integer
        // If ln(n) = value, then n = e^value
        let exp_value = value.exp(ctx.precision);
        if let Some(n) = exp_value.to_i64() {
            if n >= 2 && n <= 100 {
                let mut entry = HashMap::new();
                entry.insert("n".to_string(), Value::Number(Number::from_i64(n)));
                entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.8").unwrap()));
                result.insert("ln".to_string(), Value::Object(entry));
            }
        }

        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No simple e patterns found".to_string()));
        }

        Value::Object(result)
    }
}
