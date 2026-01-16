//! Folio Units - Physical Quantity and Unit Conversion
//!
//! Provides unit-aware quantities with dimensional analysis.
//! Supports SI, imperial, and derived units with automatic conversion.
//!
//! Categories:
//! - Length (m, km, ft, mi, etc.)
//! - Mass (kg, g, lb, oz, etc.)
//! - Time (s, min, h, d, etc.)
//! - Temperature (K, C, F, R)
//! - Current (A, mA, etc.)
//! - Amount (mol, mmol, etc.)
//! - Luminosity (cd, lm, lx)
//! - Area (m², ft², acre, etc.)
//! - Volume (L, mL, gal, etc.)
//! - Velocity (m/s, km/h, mph, etc.)
//! - Force (N, lbf, etc.)
//! - Energy (J, cal, kWh, etc.)
//! - Power (W, hp, etc.)
//! - Pressure (Pa, bar, psi, etc.)
//! - Frequency (Hz, rpm, etc.)
//! - Electrical (V, ohm, C, etc.)
//! - Data (bit, byte, MB, etc.)
//! - Angle (rad, deg, etc.)

mod dimension;
mod unit;
mod quantity;
mod convert;
mod parse;
mod units;

pub use dimension::Dimension;
pub use unit::{Unit, ConversionError};
pub use quantity::Quantity;
pub use units::UNITS;
pub use parse::{parse_unit, parse_conversion, parse_quantity_string};

use folio_plugin::PluginRegistry;

/// Load unit functions into registry
pub fn load_units_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Conversion (4 functions)
        .with_function(convert::Convert)
        .with_function(convert::ToBase)
        .with_function(convert::Simplify)
        .with_function(convert::InUnits)

        // Inspection (5 functions)
        .with_function(convert::ExtractValue)
        .with_function(convert::ExtractUnit)
        .with_function(convert::Dimensions)
        .with_function(convert::IsDimensionless)
        .with_function(convert::Compatible)

        // Construction (1 function)
        .with_function(convert::QuantityFn)
}
