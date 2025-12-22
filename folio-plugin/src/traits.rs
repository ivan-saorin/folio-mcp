//! Plugin traits

use folio_core::{Number, Value};
use crate::EvalContext;
use serde::Serialize;

/// Metadata about a function argument
#[derive(Debug, Clone, Serialize)]
pub struct ArgMeta {
    pub name: &'static str,
    pub typ: &'static str,
    pub description: &'static str,
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'static str>,
}

impl ArgMeta {
    pub const fn required(name: &'static str, typ: &'static str, description: &'static str) -> Self {
        Self { name, typ, description, optional: false, default: None }
    }
    
    pub const fn optional(name: &'static str, typ: &'static str, description: &'static str, default: &'static str) -> Self {
        Self { name, typ, description, optional: true, default: Some(default) }
    }
}

/// Metadata for a function plugin
#[derive(Debug, Clone, Serialize)]
pub struct FunctionMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub args: &'static [ArgMeta],
    pub returns: &'static str,
    pub examples: &'static [&'static str],
    pub category: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<&'static str>,
    pub related: &'static [&'static str],
}

/// Pure function plugin
pub trait FunctionPlugin: Send + Sync {
    fn meta(&self) -> FunctionMeta;
    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value;
}

/// Metadata for an analyzer plugin
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzerMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub detects: &'static [&'static str],
}

/// Pattern detection plugin
pub trait AnalyzerPlugin: Send + Sync {
    fn meta(&self) -> AnalyzerMeta;
    fn confidence(&self, value: &Number, ctx: &EvalContext) -> f64;
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value;
}

/// Metadata for a command plugin
#[derive(Debug, Clone, Serialize)]
pub struct CommandMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgMeta],
    pub examples: &'static [&'static str],
}

/// Command plugin (may have side effects)
pub trait CommandPlugin: Send + Sync {
    fn meta(&self) -> CommandMeta;
    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value;
}
