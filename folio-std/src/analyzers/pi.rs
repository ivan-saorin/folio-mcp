//! Pi pattern detection

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct PiAnalyzer;

impl AnalyzerPlugin for PiAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "pi",
            description: "Detects pi patterns",
            detects: &["π", "2π", "π²"],
        }
    }
    
    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }
    
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let pi = Number::pi(ctx.precision);
        let mut result = HashMap::new();
        
        // Test simple π multiples
        if let Ok(ratio) = value.checked_div(&pi) {
            if let Some(i) = ratio.to_i64() {
                if i.abs() <= 100 && i != 0 {
                    let mut entry = HashMap::new();
                    entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                    result.insert("π".to_string(), Value::Object(entry));
                }
            }
        }
        
        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No simple π patterns found".to_string()));
        }
        
        Value::Object(result)
    }
}
