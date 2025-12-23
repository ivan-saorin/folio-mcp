//! Plugin Registry

use crate::{FunctionPlugin, AnalyzerPlugin, CommandPlugin, FunctionMeta, CommandMeta};
use crate::EvalContext;
use folio_core::{Number, Value, FolioError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Definition of a built-in constant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDef {
    pub name: String,
    pub formula: String,
    pub source: String,
    pub category: String,
}

/// Central plugin registry
pub struct PluginRegistry {
    functions: HashMap<String, Arc<dyn FunctionPlugin>>,
    analyzers: Vec<Arc<dyn AnalyzerPlugin>>,
    commands: HashMap<String, Arc<dyn CommandPlugin>>,
    constants: HashMap<String, ConstantDef>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            analyzers: Vec::new(),
            commands: HashMap::new(),
            constants: HashMap::new(),
        }
    }
    
    pub fn with_function<F: FunctionPlugin + 'static>(mut self, f: F) -> Self {
        let name = f.meta().name.to_lowercase();
        self.functions.insert(name, Arc::new(f));
        self
    }
    
    pub fn with_analyzer<A: AnalyzerPlugin + 'static>(mut self, a: A) -> Self {
        self.analyzers.push(Arc::new(a));
        self
    }
    
    pub fn with_command<C: CommandPlugin + 'static>(mut self, c: C) -> Self {
        let name = c.meta().name.to_lowercase();
        self.commands.insert(name, Arc::new(c));
        self
    }
    
    pub fn with_constant(mut self, def: ConstantDef) -> Self {
        let name = def.name.to_lowercase();
        self.constants.insert(name, def);
        self
    }
    
    pub fn get_function(&self, name: &str) -> Option<&dyn FunctionPlugin> {
        self.functions.get(&name.to_lowercase()).map(|f| f.as_ref())
    }
    
    pub fn get_command(&self, name: &str) -> Option<&dyn CommandPlugin> {
        self.commands.get(&name.to_lowercase()).map(|c| c.as_ref())
    }
    
    pub fn get_constant(&self, name: &str) -> Option<&ConstantDef> {
        self.constants.get(&name.to_lowercase())
    }
    
    pub fn call_function(&self, name: &str, args: &[Value], ctx: &EvalContext) -> Value {
        match self.get_function(name) {
            Some(f) => f.call(args, ctx),
            None => {
                // Find similar function names for better error message
                let similar = self.find_similar_functions(name);
                let mut err = FolioError::undefined_func(name);
                if !similar.is_empty() {
                    let suggestions: Vec<&str> = similar.iter().take(5).map(|s| s.as_str()).collect();
                    err = err.with_suggestion(format!(
                        "Similar: {}. Use help() for full list.",
                        suggestions.join(", ")
                    ));
                }
                Value::Error(err)
            }
        }
    }

    /// Find function names similar to the given name (for error suggestions)
    fn find_similar_functions(&self, name: &str) -> Vec<String> {
        let name_lower = name.to_lowercase();
        let mut matches: Vec<(String, usize)> = self.functions.keys()
            .filter_map(|func_name| {
                let score = Self::similarity_score(&name_lower, func_name);
                if score > 0 {
                    Some((func_name.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity score (higher = more similar)
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.into_iter().map(|(name, _)| name).collect()
    }

    /// Calculate similarity score between two strings
    fn similarity_score(query: &str, candidate: &str) -> usize {
        let mut score = 0;

        // Exact prefix match is best
        if candidate.starts_with(query) {
            score += 100;
        }
        // Contains the query
        else if candidate.contains(query) {
            score += 50;
        }
        // Query contains the candidate
        else if query.contains(candidate) {
            score += 30;
        }

        // Levenshtein-like: count matching characters
        let query_chars: std::collections::HashSet<char> = query.chars().collect();
        let candidate_chars: std::collections::HashSet<char> = candidate.chars().collect();
        let common = query_chars.intersection(&candidate_chars).count();
        score += common * 2;

        // Penalize length difference
        let len_diff = (query.len() as i32 - candidate.len() as i32).unsigned_abs() as usize;
        if len_diff < 5 && score > 0 {
            score += 5 - len_diff;
        }

        score
    }
    
    pub fn decompose(&self, value: &Number, ctx: &EvalContext) -> Value {
        let mut result = HashMap::new();
        let threshold = 0.1;
        
        for analyzer in &self.analyzers {
            let confidence = analyzer.confidence(value, ctx);
            if confidence >= threshold {
                match analyzer.analyze(value, ctx) {
                    Value::Object(map) => { result.extend(map); }
                    Value::Error(e) => {
                        result.insert(format!("_error_{}", analyzer.meta().name), Value::Error(e));
                    }
                    other => { result.insert(analyzer.meta().name.to_string(), other); }
                }
            }
        }
        
        Value::Object(result)
    }
    
    pub fn help(&self, name: Option<&str>) -> Value {
        match name {
            Some(n) => self.help_for(n),
            None => self.general_help(),
        }
    }
    
    fn help_for(&self, name: &str) -> Value {
        let name_lower = name.to_lowercase();
        
        if let Some(f) = self.functions.get(&name_lower) {
            return Value::Object(self.function_to_help(f.meta()));
        }
        if let Some(c) = self.commands.get(&name_lower) {
            return Value::Object(self.command_to_help(c.meta()));
        }
        if let Some(c) = self.constants.get(&name_lower) {
            return Value::Object(self.constant_to_help(c));
        }
        
        Value::Error(FolioError::new("NOT_FOUND", format!("No function, command, or constant named '{}'", name)))
    }
    
    fn general_help(&self) -> Value {
        let mut help = HashMap::new();
        
        let mut funcs_by_cat: HashMap<String, Vec<String>> = HashMap::new();
        for (name, f) in &self.functions {
            let cat = f.meta().category.to_string();
            funcs_by_cat.entry(cat).or_default().push(name.clone());
        }
        help.insert("functions".to_string(), 
            Value::Object(funcs_by_cat.into_iter()
                .map(|(k, v)| (k, Value::List(v.into_iter().map(Value::Text).collect())))
                .collect()));
        
        help.insert("constants".to_string(),
            Value::List(self.constants.keys().cloned().map(Value::Text).collect()));
        
        help.insert("commands".to_string(),
            Value::List(self.commands.keys().cloned().map(Value::Text).collect()));
        
        help.insert("usage".to_string(), 
            Value::Text("Call help('function_name') for detailed help.".to_string()));
        
        Value::Object(help)
    }
    
    fn function_to_help(&self, meta: FunctionMeta) -> HashMap<String, Value> {
        let mut help = HashMap::new();
        help.insert("name".to_string(), Value::Text(meta.name.to_string()));
        help.insert("type".to_string(), Value::Text("function".to_string()));
        help.insert("description".to_string(), Value::Text(meta.description.to_string()));
        help.insert("usage".to_string(), Value::Text(meta.usage.to_string()));
        help.insert("returns".to_string(), Value::Text(meta.returns.to_string()));
        help.insert("category".to_string(), Value::Text(meta.category.to_string()));
        help.insert("args".to_string(), Value::List(
            meta.args.iter().map(|a| {
                let mut arg = HashMap::new();
                arg.insert("name".to_string(), Value::Text(a.name.to_string()));
                arg.insert("type".to_string(), Value::Text(a.typ.to_string()));
                arg.insert("description".to_string(), Value::Text(a.description.to_string()));
                arg.insert("optional".to_string(), Value::Bool(a.optional));
                Value::Object(arg)
            }).collect()
        ));
        help.insert("examples".to_string(), Value::List(
            meta.examples.iter().map(|e| Value::Text(e.to_string())).collect()
        ));
        help
    }
    
    fn command_to_help(&self, meta: CommandMeta) -> HashMap<String, Value> {
        let mut help = HashMap::new();
        help.insert("name".to_string(), Value::Text(meta.name.to_string()));
        help.insert("type".to_string(), Value::Text("command".to_string()));
        help.insert("description".to_string(), Value::Text(meta.description.to_string()));
        help
    }
    
    fn constant_to_help(&self, def: &ConstantDef) -> HashMap<String, Value> {
        let mut help = HashMap::new();
        help.insert("name".to_string(), Value::Text(def.name.clone()));
        help.insert("type".to_string(), Value::Text("constant".to_string()));
        help.insert("formula".to_string(), Value::Text(def.formula.clone()));
        help.insert("source".to_string(), Value::Text(def.source.clone()));
        help
    }
    
    pub fn list_functions(&self, category: Option<&str>) -> Value {
        let funcs: Vec<Value> = self.functions.values()
            .filter(|f| category.map_or(true, |c| f.meta().category == c))
            .map(|f| {
                let meta = f.meta();
                let mut obj = HashMap::new();
                obj.insert("name".to_string(), Value::Text(meta.name.to_string()));
                obj.insert("description".to_string(), Value::Text(meta.description.to_string()));
                obj.insert("usage".to_string(), Value::Text(meta.usage.to_string()));
                obj.insert("category".to_string(), Value::Text(meta.category.to_string()));
                Value::Object(obj)
            })
            .collect();
        Value::List(funcs)
    }
    
    pub fn list_constants(&self) -> Value {
        let consts: Vec<Value> = self.constants.values()
            .map(|c| {
                let mut obj = HashMap::new();
                obj.insert("name".to_string(), Value::Text(c.name.clone()));
                obj.insert("formula".to_string(), Value::Text(c.formula.clone()));
                obj.insert("source".to_string(), Value::Text(c.source.clone()));
                Value::Object(obj)
            })
            .collect();
        Value::List(consts)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
