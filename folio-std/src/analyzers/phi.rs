//! Golden ratio pattern detection

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct PhiAnalyzer;

impl AnalyzerPlugin for PhiAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "phi",
            description: "Detects golden ratio patterns",
            detects: &["φ", "φ²", "1/φ"],
        }
    }
    
    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }
    
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let phi = Number::phi(ctx.precision);
        let mut result = HashMap::new();
        
        // Test φ^n for small n
        for n in -3i32..=5 {
            let phi_n = phi.pow(n);
            if let Ok(ratio) = value.checked_div(&phi_n) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 {
                        let key = format!("φ^{}", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.9").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }
        
        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No simple φ patterns found".to_string()));
        }
        
        Value::Object(result)
    }
}
