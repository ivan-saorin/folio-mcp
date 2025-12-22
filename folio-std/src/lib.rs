//! Folio Standard Library

pub mod functions;
pub mod analyzers;
pub mod commands;
pub mod constants;

use folio_plugin::PluginRegistry;

/// Load standard library into registry
pub fn load_standard_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        .with_function(functions::Sqrt)
        .with_function(functions::Ln)
        .with_function(functions::Exp)
        .with_function(functions::Pow)
        .with_function(functions::Abs)
        .with_function(functions::Sin)
        .with_function(functions::Cos)
        .with_function(functions::Tan)
        .with_function(functions::Sum)
        .with_function(functions::Round)
        .with_function(functions::Floor)
        .with_function(functions::Ceil)
        .with_analyzer(analyzers::PhiAnalyzer)
        .with_analyzer(analyzers::PiAnalyzer)
        .with_analyzer(analyzers::EAnalyzer)
        .with_command(commands::Trace)
        .with_command(commands::Explain)
        .with_constant(constants::phi())
        .with_constant(constants::pi())
        .with_constant(constants::e())
}

/// Create registry with standard library
pub fn standard_registry() -> PluginRegistry {
    load_standard_library(PluginRegistry::new())
}
