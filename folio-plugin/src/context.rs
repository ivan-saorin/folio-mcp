//! Evaluation Context

use folio_core::Value;
use crate::PluginRegistry;
use std::collections::HashMap;
use std::sync::Arc;

/// Evaluation context passed to plugins
pub struct EvalContext {
    pub precision: u32,
    pub variables: HashMap<String, Value>,
    pub registry: Arc<PluginRegistry>,
    pub tracing: bool,
    pub trace: Vec<TraceStep>,
}

/// Single step in evaluation trace
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub cell: String,
    pub formula: String,
    pub result: Value,
    pub dependencies: Vec<String>,
}

impl EvalContext {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            precision: 50,
            variables: HashMap::new(),
            registry,
            tracing: false,
            trace: Vec::new(),
        }
    }
    
    pub fn with_precision(mut self, precision: u32) -> Self {
        self.precision = precision;
        self
    }
    
    pub fn with_variables(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables = vars;
        self
    }
    
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing = enabled;
        self
    }
    
    pub fn get_var(&self, name: &str) -> Value {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.is_empty() {
            return Value::Error(folio_core::FolioError::undefined_var(name));
        }

        let root = match self.variables.get(parts[0]) {
            Some(v) => v.clone(),
            None => {
                // Check if it's a registered constant (π, φ, e)
                if let Some(constant) = self.registry.get_constant(parts[0]) {
                    // Evaluate the constant's formula
                    // For built-in constants, the formula is a function call like "pi", "exp(1)", "(1 + sqrt(5)) / 2"
                    return self.eval_constant_formula(&constant.formula);
                }
                return Value::Error(folio_core::FolioError::undefined_var(parts[0]));
            }
        };

        let mut current = root;
        for part in &parts[1..] {
            current = current.get(part);
            if current.is_error() {
                return current;
            }
        }
        current
    }

    /// Evaluate a constant's formula (e.g., "pi", "exp(1)", "(1 + sqrt(5)) / 2", "0.51099895")
    fn eval_constant_formula(&self, formula: &str) -> Value {
        // First try: parse as numeric literal (handles "0.51099895", "1776.86", "299792458", etc.)
        if let Ok(n) = folio_core::Number::from_str(formula) {
            return Value::Number(n);
        }

        // Handle special computed formulas
        match formula {
            "pi" => Value::Number(folio_core::Number::pi(self.precision)),
            "exp(1)" => Value::Number(folio_core::Number::e(self.precision)),
            "(1 + sqrt(5)) / 2" => Value::Number(folio_core::Number::phi(self.precision)),
            "sqrt(2)" => {
                let two = folio_core::Number::from_i64(2);
                two.sqrt(self.precision)
                    .map(Value::Number)
                    .unwrap_or_else(|e| Value::Error(e.into()))
            }
            "sqrt(3)" => {
                let three = folio_core::Number::from_i64(3);
                three.sqrt(self.precision)
                    .map(Value::Number)
                    .unwrap_or_else(|e| Value::Error(e.into()))
            }
            _ => {
                // Unknown formula
                Value::Error(folio_core::FolioError::new("UNKNOWN_CONSTANT",
                    format!("Unknown constant formula: {}", formula)))
            }
        }
    }
    
    pub fn set_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    
    pub fn record_trace(&mut self, cell: String, formula: String, result: Value, dependencies: Vec<String>) {
        if self.tracing {
            self.trace.push(TraceStep { cell, formula, result, dependencies });
        }
    }
}
