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
        // Mathematical constants
        .with_constant(constants::phi())
        .with_constant(constants::pi())
        .with_constant(constants::e())
        .with_constant(constants::sqrt2())
        .with_constant(constants::sqrt3())
        // Particle masses (MeV)
        .with_constant(constants::m_e())
        .with_constant(constants::m_mu())
        .with_constant(constants::m_tau())
        .with_constant(constants::m_higgs())
        // CKM matrix elements
        .with_constant(constants::v_us())
        .with_constant(constants::v_cb())
        .with_constant(constants::v_ub())
        .with_constant(constants::v_ts())
        // Physical constants
        .with_constant(constants::c())
        .with_constant(constants::alpha())
        // ASCII aliases for Unicode constants
        .with_constant(constants::phi_ascii())    // "phi" alias for "φ"
        .with_constant(constants::pi_ascii())     // "pi" alias for "π"
        .with_constant(constants::alpha_ascii())  // "alpha" alias for "α"
        .with_constant(constants::m_mu_ascii())   // "m_mu" alias for "m_μ"
        .with_constant(constants::m_tau_ascii())  // "m_tau" alias for "m_τ"
}

/// Create registry with standard library
pub fn standard_registry() -> PluginRegistry {
    load_standard_library(PluginRegistry::new())
}
