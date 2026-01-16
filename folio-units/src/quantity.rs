//! Quantity type - a value with an associated unit

use std::fmt;
use serde::{Serialize, Deserialize};
use folio_core::Number;
use crate::{Unit, Dimension};
use crate::unit::ConversionError;

/// A physical quantity: a numeric value with an associated unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    /// The numeric value
    pub value: Number,
    /// The unit of measurement
    pub unit: Unit,
}

impl Quantity {
    /// Create a new quantity
    pub fn new(value: Number, unit: Unit) -> Self {
        Quantity { value, unit }
    }

    /// Create a dimensionless quantity (pure number)
    pub fn dimensionless(value: Number) -> Self {
        Quantity {
            value,
            unit: Unit::new("", "dimensionless", Dimension::DIMENSIONLESS, Number::from_i64(1), "dimensionless"),
        }
    }

    /// Get the dimension of this quantity
    pub fn dimension(&self) -> Dimension {
        self.unit.dimension
    }

    /// Check if this is a dimensionless quantity
    pub fn is_dimensionless(&self) -> bool {
        self.unit.dimension.is_dimensionless()
    }

    /// Check if two quantities have compatible dimensions
    pub fn is_compatible(&self, other: &Quantity) -> bool {
        self.unit.is_compatible(&other.unit)
    }

    /// Convert to SI base units
    pub fn to_si(&self) -> Quantity {
        let si_value = self.unit.to_si(&self.value);
        let si_unit = self.create_si_unit();
        Quantity::new(si_value, si_unit)
    }

    /// Convert to another unit
    pub fn convert_to(&self, target: &Unit, precision: u32) -> Result<Quantity, ConversionError> {
        let new_value = self.unit.convert_to(&self.value, target, precision)?;
        Ok(Quantity::new(new_value, target.clone()))
    }

    /// Extract the numeric value
    pub fn extract_value(&self) -> Number {
        self.value.clone()
    }

    /// Extract the unit
    pub fn extract_unit(&self) -> Unit {
        self.unit.clone()
    }

    /// Get the value in SI base units
    pub fn si_value(&self) -> Number {
        self.unit.to_si(&self.value)
    }

    /// Add two quantities (must have compatible dimensions)
    pub fn add(&self, other: &Quantity, precision: u32) -> Result<Quantity, ConversionError> {
        if !self.is_compatible(other) {
            return Err(ConversionError::IncompatibleDimensions {
                from: other.unit.symbol.clone(),
                to: self.unit.symbol.clone(),
                from_dim: other.unit.dimension,
                to_dim: self.unit.dimension,
            });
        }

        // Convert other to same unit as self, then add
        let converted = other.convert_to(&self.unit, precision)?;
        Ok(Quantity::new(self.value.add(&converted.value), self.unit.clone()))
    }

    /// Subtract two quantities (must have compatible dimensions)
    pub fn sub(&self, other: &Quantity, precision: u32) -> Result<Quantity, ConversionError> {
        if !self.is_compatible(other) {
            return Err(ConversionError::IncompatibleDimensions {
                from: other.unit.symbol.clone(),
                to: self.unit.symbol.clone(),
                from_dim: other.unit.dimension,
                to_dim: self.unit.dimension,
            });
        }

        let converted = other.convert_to(&self.unit, precision)?;
        Ok(Quantity::new(self.value.sub(&converted.value), self.unit.clone()))
    }

    /// Multiply two quantities (dimensions are multiplied)
    pub fn mul(&self, other: &Quantity) -> Quantity {
        let new_value = self.value.mul(&other.value);
        let new_unit = self.unit.multiply(&other.unit);
        Quantity::new(new_value, new_unit)
    }

    /// Divide two quantities (dimensions are divided)
    pub fn div(&self, other: &Quantity, precision: u32) -> Result<Quantity, ConversionError> {
        let new_value = self.value.checked_div(&other.value)
            .map_err(|e| ConversionError::NumberError(e))?;
        let new_unit = self.unit.divide(&other.unit, precision)
            .map_err(|e| ConversionError::NumberError(e))?;
        Ok(Quantity::new(new_value, new_unit))
    }

    /// Raise quantity to an integer power
    pub fn pow(&self, exp: i32, precision: u32) -> Quantity {
        let new_value = self.value.pow(exp);
        let new_unit = self.unit.power(exp, precision);
        Quantity::new(new_value, new_unit)
    }

    /// Take the square root (dimension exponents must be even)
    pub fn sqrt(&self, precision: u32) -> Result<Quantity, ConversionError> {
        // Check that all dimension exponents are even
        for &exp in &self.unit.dimension.exponents {
            if exp % 2 != 0 {
                return Err(ConversionError::IncompatibleDimensions {
                    from: self.unit.symbol.clone(),
                    to: "sqrt".to_string(),
                    from_dim: self.unit.dimension,
                    to_dim: Dimension::DIMENSIONLESS,
                });
            }
        }

        let new_value = self.value.sqrt(precision)
            .map_err(|e| ConversionError::NumberError(e))?;

        // Halve all dimension exponents
        let mut new_exponents = [0i32; 7];
        for i in 0..7 {
            new_exponents[i] = self.unit.dimension.exponents[i] / 2;
        }
        let new_dimension = Dimension::new(new_exponents);

        // Calculate new SI factor
        let new_factor = self.unit.to_si_factor.sqrt(precision)
            .map_err(|e| ConversionError::NumberError(e))?;

        let new_unit = Unit::new(
            &format!("√{}", self.unit.symbol),
            &format!("square root of {}", self.unit.name),
            new_dimension,
            new_factor,
            &self.unit.category,
        );

        Ok(Quantity::new(new_value, new_unit))
    }

    /// Create the SI base unit for this quantity's dimension
    fn create_si_unit(&self) -> Unit {
        let dim = self.unit.dimension;
        let symbol = format!("{}", dim);
        let name = dim.name().map(|s| s.to_string()).unwrap_or_else(|| symbol.clone());

        Unit::new(&symbol, &name, dim, Number::from_i64(1), "si_base")
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unit.symbol.is_empty() {
            write!(f, "{}", self.value)
        } else {
            write!(f, "{} {}", self.value, self.unit.symbol)
        }
    }
}

impl PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        // Compare SI values for equality
        if !self.is_compatible(other) {
            return false;
        }
        self.si_value() == other.si_value()
    }
}

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
    fn test_quantity_creation() {
        let q = Quantity::new(Number::from_i64(5), meter());
        assert_eq!(q.value, Number::from_i64(5));
        assert_eq!(q.unit.symbol, "m");
    }

    #[test]
    fn test_dimensionless() {
        let q = Quantity::dimensionless(Number::from_i64(42));
        assert!(q.is_dimensionless());
    }

    #[test]
    fn test_to_si() {
        let q = Quantity::new(Number::from_i64(5), kilometer());
        let si = q.to_si();
        assert_eq!(si.value, Number::from_i64(5000));
    }

    #[test]
    fn test_convert_to() {
        let q = Quantity::new(Number::from_i64(5000), meter());
        let converted = q.convert_to(&kilometer(), 50).unwrap();
        assert_eq!(converted.value, Number::from_i64(5));
    }

    #[test]
    fn test_add() {
        let q1 = Quantity::new(Number::from_i64(1), kilometer());
        let q2 = Quantity::new(Number::from_i64(500), meter());
        let sum = q1.add(&q2, 50).unwrap();

        // 1 km + 500 m = 1.5 km
        let expected = Number::from_str("1.5").unwrap();
        assert_eq!(sum.value, expected);
        assert_eq!(sum.unit.symbol, "km");
    }

    #[test]
    fn test_mul() {
        let length = Quantity::new(Number::from_i64(5), meter());
        let width = Quantity::new(Number::from_i64(3), meter());
        let area = length.mul(&width);

        assert_eq!(area.value, Number::from_i64(15));
        assert_eq!(area.dimension(), Dimension::AREA);
    }

    #[test]
    fn test_div() {
        let distance = Quantity::new(Number::from_i64(100), meter());
        let time = Quantity::new(Number::from_i64(10), second());
        let velocity = distance.div(&time, 50).unwrap();

        assert_eq!(velocity.value, Number::from_i64(10));
        assert_eq!(velocity.dimension(), Dimension::VELOCITY);
    }

    #[test]
    fn test_pow() {
        let length = Quantity::new(Number::from_i64(5), meter());
        let volume = length.pow(3, 50);

        assert_eq!(volume.value, Number::from_i64(125));
        assert_eq!(volume.dimension(), Dimension::VOLUME);
    }

    #[test]
    fn test_equality() {
        let q1 = Quantity::new(Number::from_i64(1), kilometer());
        let q2 = Quantity::new(Number::from_i64(1000), meter());

        assert_eq!(q1, q2);
    }

    #[test]
    fn test_display() {
        let q = Quantity::new(Number::from_i64(5), meter());
        let display = format!("{}", q);
        // Number displays with decimals, so check it contains the right parts
        assert!(display.contains("5"));
        assert!(display.contains("m"));
    }
}
