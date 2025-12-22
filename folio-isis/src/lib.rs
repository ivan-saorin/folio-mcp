//! ISIS Formula Extensions for Folio
//!
//! Provides:
//! - ISIS transform function
//! - ISIS inverse transform
//! - Specialized φ analyzers for error archaeology
//! - GBR (Geometric Background Radiation) analysis

mod transform;
mod analyzers;

use folio_plugin::PluginRegistry;

pub use transform::{IsisTransform, IsisInverse};
pub use analyzers::{PhillipAnalyzer, ErrorArchaeologist};

/// Load ISIS extensions into registry
pub fn load_isis_extensions(registry: PluginRegistry) -> PluginRegistry {
    registry
        .with_function(IsisTransform)
        .with_function(IsisInverse)
        .with_analyzer(PhillipAnalyzer)
}
