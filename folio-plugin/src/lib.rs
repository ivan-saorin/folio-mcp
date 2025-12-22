//! Folio Plugin System
//!
//! Provides traits for extending Folio with custom:
//! - Functions (pure computation)
//! - Analyzers (pattern detection)
//! - Commands (side effects)

mod traits;
mod registry;
mod context;

pub use traits::{
    FunctionPlugin, FunctionMeta,
    AnalyzerPlugin, AnalyzerMeta,
    CommandPlugin, CommandMeta,
    ArgMeta,
};
pub use registry::{PluginRegistry, ConstantDef};
pub use context::{EvalContext, TraceStep};

/// Re-export core types for plugin authors
pub mod prelude {
    pub use crate::{
        FunctionPlugin, FunctionMeta,
        AnalyzerPlugin, AnalyzerMeta,
        CommandPlugin, CommandMeta,
        ArgMeta, PluginRegistry, EvalContext, TraceStep,
    };
    pub use folio_core::prelude::*;
}
