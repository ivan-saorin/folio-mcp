//! ISIS-specific analyzers
//!
//! Specialized pattern detection for Phillip formula research.

use folio_plugin::prelude::*;
use std::collections::HashMap;

/// Advanced φ analyzer for Phillip formula research
/// Detects combinations of φ powers with other constants
pub struct PhillipAnalyzer;

impl AnalyzerPlugin for PhillipAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "phillip",
            description: "Deep φ analysis for ISIS formula research",
            detects: &["φ^n", "ln(φ)", "φ^n × ln(φ)", "φ^n × π", "Fibonacci", "Lucas"],
        }
    }

    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.9 // Always try for research tooling
    }

    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let ln_phi = phi.ln(ctx.precision).unwrap_or(Number::from_i64(0));

        let mut result = HashMap::new();

        // Test φ^n patterns for larger range
        for n in -10i32..=10 {
            let phi_n = phi.pow(n);
            if let Ok(ratio) = value.checked_div(&phi_n) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 1000 && i != 0 {
                        let key = format!("φ^{}", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("power".to_string(), Value::Number(Number::from_i64(n as i64)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.95").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Test φ^n × ln(φ) patterns
        for n in -5i32..=5 {
            let phi_n = phi.pow(n);
            let phi_n_ln_phi = phi_n.mul(&ln_phi);
            if let Ok(ratio) = value.checked_div(&phi_n_ln_phi) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 && i != 0 {
                        let key = format!("φ^{}×ln(φ)", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("phi_power".to_string(), Value::Number(Number::from_i64(n as i64)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.9").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Test φ^n × π patterns
        for n in -5i32..=5 {
            let phi_n = phi.pow(n);
            let phi_n_pi = phi_n.mul(&pi);
            if let Ok(ratio) = value.checked_div(&phi_n_pi) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 && i != 0 {
                        let key = format!("φ^{}×π", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("phi_power".to_string(), Value::Number(Number::from_i64(n as i64)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.85").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Check Fibonacci numbers (1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, ...)
        let fibonacci: [i64; 15] = [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610];
        if let Some(val_i64) = value.to_i64() {
            if val_i64 > 0 {
                if let Some(pos) = fibonacci.iter().position(|&f| f == val_i64) {
                    let mut entry = HashMap::new();
                    entry.insert("index".to_string(), Value::Number(Number::from_i64((pos + 1) as i64)));
                    entry.insert("confidence".to_string(), Value::Number(Number::from_str("1.0").unwrap()));
                    result.insert("Fibonacci".to_string(), Value::Object(entry));
                }
            }
        }

        // Check Lucas numbers (2, 1, 3, 4, 7, 11, 18, 29, 47, 76, 123, ...)
        let lucas: [i64; 12] = [2, 1, 3, 4, 7, 11, 18, 29, 47, 76, 123, 199];
        if let Some(val_i64) = value.to_i64() {
            if val_i64 > 0 {
                if let Some(pos) = lucas.iter().position(|&l| l == val_i64) {
                    let mut entry = HashMap::new();
                    entry.insert("index".to_string(), Value::Number(Number::from_i64(pos as i64)));
                    entry.insert("confidence".to_string(), Value::Number(Number::from_str("1.0").unwrap()));
                    result.insert("Lucas".to_string(), Value::Object(entry));
                }
            }
        }

        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No φ-related patterns found".to_string()));
        }

        Value::Object(result)
    }
}

/// Error archaeologist - recursive error analysis
/// Analyzes how far a value is from known constants and looks for φ structure in errors
pub struct ErrorArchaeologist;

impl AnalyzerPlugin for ErrorArchaeologist {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "archaeology",
            description: "Recursive error decomposition looking for hidden φ structure",
            detects: &["nested φ", "error chains", "convergent series"],
        }
    }

    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }

    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let e = Number::e(ctx.precision);
        let one = Number::from_i64(1);

        let mut result = HashMap::new();
        let mut layers = Vec::new();

        // Layer 0: Original value
        let mut current = value.clone();
        let max_depth = 5;

        for depth in 0..max_depth {
            let mut layer_info = HashMap::new();
            layer_info.insert("depth".to_string(), Value::Number(Number::from_i64(depth as i64)));
            layer_info.insert("value".to_string(), Value::Number(current.clone()));

            // Compute errors from key constants
            let error_phi = current.sub(&phi);
            let error_one = current.sub(&one);
            let error_pi = current.sub(&pi);
            let error_e = current.sub(&e);

            let mut errors = HashMap::new();
            errors.insert("φ".to_string(), Value::Number(error_phi.clone()));
            errors.insert("1".to_string(), Value::Number(error_one.clone()));
            errors.insert("π".to_string(), Value::Number(error_pi));
            errors.insert("e".to_string(), Value::Number(error_e));
            layer_info.insert("errors".to_string(), Value::Object(errors));

            // Check if any error is a simple φ multiple
            let mut found_phi_structure = false;
            for n in -5i32..=5 {
                if n == 0 {
                    continue;
                }
                let phi_n = phi.pow(n);
                if let Ok(ratio) = error_phi.checked_div(&phi_n) {
                    if let Some(i) = ratio.to_i64() {
                        if i.abs() <= 10 && i != 0 {
                            let mut phi_match = HashMap::new();
                            phi_match.insert("power".to_string(), Value::Number(Number::from_i64(n as i64)));
                            phi_match.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                            layer_info.insert("phi_structure".to_string(), Value::Object(phi_match));
                            found_phi_structure = true;
                            break;
                        }
                    }
                }
            }

            layers.push(Value::Object(layer_info));

            // If we found exact φ structure or error is very small, stop
            if found_phi_structure || error_phi.abs().as_decimal(15).starts_with("0.000000000") {
                break;
            }

            // Next layer: analyze the error from φ
            current = error_phi;
        }

        result.insert("layers".to_string(), Value::List(layers));
        result.insert("depth_analyzed".to_string(), Value::Number(Number::from_i64(max_depth as i64)));

        Value::Object(result)
    }
}
