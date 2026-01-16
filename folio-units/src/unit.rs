//! Unit representation with conversion factors

use std::fmt;
use serde::{Serialize, Deserialize};
use folio_core::Number;
use crate::Dimension;

/// Represents a physical unit with its dimension and conversion factors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// The unit symbol (e.g., "m", "kg", "s")
    pub symbol: String,
    /// The unit name (e.g., "meter", "kilogram", "second")
    pub name: String,
    /// The dimensional signature
    pub dimension: Dimension,
    /// Factor to convert to SI base unit (value_si = value * to_si_factor + to_si_offset)
    pub to_si_factor: Number,
    /// Offset for non-proportional units like temperature (Celsius, Fahrenheit)
    pub to_si_offset: Number,
    /// Category for organization (e.g., "length", "mass", "time")
    pub category: String,
}

impl Unit {
    /// Create a new unit with proportional conversion (no offset)
    pub fn new(
        symbol: &str,
        name: &str,
        dimension: Dimension,
        to_si_factor: Number,
        category: &str,
    ) -> Self {
        Unit {
            symbol: symbol.to_string(),
            name: name.to_string(),
            dimension,
            to_si_factor,
            to_si_offset: Number::from_i64(0),
            category: category.to_string(),
        }
    }

    /// Create a unit with offset (for temperature conversions)
    pub fn with_offset(
        symbol: &str,
        name: &str,
        dimension: Dimension,
        to_si_factor: Number,
        to_si_offset: Number,
        category: &str,
    ) -> Self {
        Unit {
            symbol: symbol.to_string(),
            name: name.to_string(),
            dimension,
            to_si_factor,
            to_si_offset,
            category: category.to_string(),
        }
    }

    /// Check if this is a base SI unit
    pub fn is_si_base(&self) -> bool {
        self.to_si_factor == Number::from_i64(1) && self.to_si_offset.is_zero()
    }

    /// Check if this unit has an offset (non-proportional conversion)
    pub fn has_offset(&self) -> bool {
        !self.to_si_offset.is_zero()
    }

    /// Check if two units are dimensionally compatible (can be converted)
    pub fn is_compatible(&self, other: &Unit) -> bool {
        self.dimension == other.dimension
    }

    /// Convert a value from this unit to SI base unit
    pub fn to_si(&self, value: &Number) -> Number {
        // value_si = value * factor + offset
        value.mul(&self.to_si_factor).add(&self.to_si_offset)
    }

    /// Convert a value from SI base unit to this unit
    pub fn from_si(&self, value_si: &Number, precision: u32) -> Result<Number, folio_core::NumberError> {
        // value = (value_si - offset) / factor
        let shifted = value_si.sub(&self.to_si_offset);
        shifted.checked_div(&self.to_si_factor)
    }

    /// Convert a value from this unit to another unit
    pub fn convert_to(&self, value: &Number, target: &Unit, precision: u32) -> Result<Number, ConversionError> {
        if !self.is_compatible(target) {
            return Err(ConversionError::IncompatibleDimensions {
                from: self.symbol.clone(),
                to: target.symbol.clone(),
                from_dim: self.dimension,
                to_dim: target.dimension,
            });
        }

        // Convert to SI, then from SI to target
        let si_value = self.to_si(value);
        target.from_si(&si_value, precision)
            .map_err(|e| ConversionError::NumberError(e))
    }

    /// Get the inverse unit (e.g., Hz -> s)
    pub fn inverse(&self, precision: u32) -> Result<Unit, folio_core::NumberError> {
        let one = Number::from_i64(1);
        let inv_factor = one.checked_div(&self.to_si_factor)?;

        Ok(Unit {
            symbol: format!("1/{}", self.symbol),
            name: format!("inverse {}", self.name),
            dimension: self.dimension.invert(),
            to_si_factor: inv_factor,
            to_si_offset: Number::from_i64(0), // Inverse of offset unit is complex
            category: self.category.clone(),
        })
    }

    /// Multiply two units (e.g., m * m -> m^2)
    pub fn multiply(&self, other: &Unit) -> Unit {
        Unit {
            symbol: format!("{}·{}", self.symbol, other.symbol),
            name: format!("{} {}", self.name, other.name),
            dimension: self.dimension.multiply(&other.dimension),
            to_si_factor: self.to_si_factor.mul(&other.to_si_factor),
            to_si_offset: Number::from_i64(0), // Product of offset units loses meaning
            category: "derived".to_string(),
        }
    }

    /// Divide two units (e.g., m / s -> m/s)
    pub fn divide(&self, other: &Unit, precision: u32) -> Result<Unit, folio_core::NumberError> {
        let factor = self.to_si_factor.checked_div(&other.to_si_factor)?;

        Ok(Unit {
            symbol: format!("{}/{}", self.symbol, other.symbol),
            name: format!("{} per {}", self.name, other.name),
            dimension: self.dimension.divide(&other.dimension),
            to_si_factor: factor,
            to_si_offset: Number::from_i64(0),
            category: "derived".to_string(),
        })
    }

    /// Raise unit to a power (e.g., m^2, m^3)
    pub fn power(&self, exp: i32, _precision: u32) -> Unit {
        let new_factor = self.to_si_factor.pow(exp);

        let symbol = if exp == 1 {
            self.symbol.clone()
        } else {
            format!("{}^{}", self.symbol, exp)
        };

        Unit {
            symbol,
            name: format!("{} to the {}", self.name, exp),
            dimension: self.dimension.power(exp),
            to_si_factor: new_factor,
            to_si_offset: Number::from_i64(0),
            category: self.category.clone(),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

/// Errors that can occur during unit conversion
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Units have incompatible dimensions
    IncompatibleDimensions {
        from: String,
        to: String,
        from_dim: Dimension,
        to_dim: Dimension,
    },
    /// Unknown unit symbol
    UnknownUnit(String),
    /// Numeric error during conversion
    NumberError(folio_core::NumberError),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::IncompatibleDimensions { from, to, from_dim, to_dim } => {
                write!(f, "cannot convert {} ({}) to {} ({}): incompatible dimensions",
                    from, from_dim, to, to_dim)
            }
            ConversionError::UnknownUnit(unit) => {
                write!(f, "unknown unit: {}", unit)
            }
            ConversionError::NumberError(e) => {
                write!(f, "numeric error: {}", e)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn meter() -> Unit {
        Unit::new("m", "meter", Dimension::LENGTH, Number::from_i64(1), "length")
    }

    fn kilometer() -> Unit {
        Unit::new("km", "kilometer", Dimension::LENGTH, Number::from_i64(1000), "length")
    }

    fn second() -> Unit {
        Unit::new("s", "second", Dimension::TIME, Number::from_i64(1), "time")
    }

    #[test]
    fn test_si_base_unit() {
        let m = meter();
        assert!(m.is_si_base());

        let km = kilometer();
        assert!(!km.is_si_base());
    }

    #[test]
    fn test_compatible_units() {
        let m = meter();
        let km = kilometer();
        let s = second();

        assert!(m.is_compatible(&km));
        assert!(!m.is_compatible(&s));
    }

    #[test]
    fn test_to_si_conversion() {
        let km = kilometer();
        let value = Number::from_i64(5);
        let si_value = km.to_si(&value);

        assert_eq!(si_value, Number::from_i64(5000));
    }

    #[test]
    fn test_from_si_conversion() {
        let km = kilometer();
        let si_value = Number::from_i64(5000);
        let value = km.from_si(&si_value, 50).unwrap();

        assert_eq!(value, Number::from_i64(5));
    }

    #[test]
    fn test_unit_conversion() {
        let m = meter();
        let km = kilometer();

        // Convert 5000 m to km
        let value = Number::from_i64(5000);
        let converted = m.convert_to(&value, &km, 50).unwrap();

        assert_eq!(converted, Number::from_i64(5));
    }

    #[test]
    fn test_unit_power() {
        let m = meter();
        let m2 = m.power(2, 50);

        assert_eq!(m2.symbol, "m^2");
        assert_eq!(m2.dimension, Dimension::AREA);
    }

    #[test]
    fn test_unit_multiply() {
        let m = meter();
        let m2 = m.multiply(&m);

        assert_eq!(m2.dimension, Dimension::AREA);
    }

    #[test]
    fn test_unit_divide() {
        let m = meter();
        let s = second();
        let velocity = m.divide(&s, 50).unwrap();

        assert_eq!(velocity.dimension, Dimension::VELOCITY);
    }
}
