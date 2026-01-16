//! Unit string parsing - parse expressions like "km/h" or "m^2"

use folio_core::Number;
use crate::{Unit, Dimension};
use crate::unit::ConversionError;
use crate::units::UNITS;

/// Parse a unit string into a Unit
///
/// Supported formats:
/// - Simple: "m", "kg", "s"
/// - Powers: "m^2", "s^-1"
/// - Products: "m*s", "kg*m"
/// - Quotients: "m/s", "kg/m^2"
/// - Combined: "kg*m/s^2", "m^2*kg/s^3"
pub fn parse_unit(s: &str) -> Result<Unit, ConversionError> {
    let s = s.trim();

    if s.is_empty() {
        // Empty string = dimensionless
        return Ok(Unit::new("", "dimensionless", Dimension::DIMENSIONLESS, Number::from_i64(1), "dimensionless"));
    }

    // Try simple lookup first
    if let Some(unit) = UNITS.get(s) {
        return Ok(unit.clone());
    }

    // Parse complex expression
    parse_unit_expression(s)
}

/// Parse a complex unit expression like "kg*m/s^2"
fn parse_unit_expression(s: &str) -> Result<Unit, ConversionError> {
    // Split by '/' to handle quotients
    let parts: Vec<&str> = s.splitn(2, '/').collect();

    let numerator = parse_product(parts[0])?;

    if parts.len() == 1 {
        return Ok(numerator);
    }

    let denominator = parse_product(parts[1])?;

    // Divide numerator by denominator
    numerator.divide(&denominator, 50)
        .map_err(|e| ConversionError::NumberError(e))
}

/// Parse a product of units like "kg*m" or "m^2*s"
fn parse_product(s: &str) -> Result<Unit, ConversionError> {
    let s = s.trim();

    if s.is_empty() {
        return Ok(Unit::new("1", "dimensionless", Dimension::DIMENSIONLESS, Number::from_i64(1), "dimensionless"));
    }

    // Split by '*' or '·' or ' '
    let factors: Vec<&str> = s.split(|c| c == '*' || c == '·' || c == ' ')
        .filter(|p| !p.is_empty())
        .collect();

    if factors.is_empty() {
        return Ok(Unit::new("1", "dimensionless", Dimension::DIMENSIONLESS, Number::from_i64(1), "dimensionless"));
    }

    let mut result = parse_power(factors[0])?;

    for factor in &factors[1..] {
        let unit = parse_power(factor)?;
        result = result.multiply(&unit);
    }

    Ok(result)
}

/// Parse a unit with optional power like "m^2" or "s^-1"
fn parse_power(s: &str) -> Result<Unit, ConversionError> {
    let s = s.trim();

    // Check for power notation
    if let Some(caret_pos) = s.find('^') {
        let base = &s[..caret_pos];
        let exp_str = &s[caret_pos + 1..];

        let base_unit = lookup_base_unit(base)?;
        let exponent: i32 = exp_str.parse()
            .map_err(|_| ConversionError::UnknownUnit(format!("invalid exponent: {}", exp_str)))?;

        return Ok(base_unit.power(exponent, 50));
    }

    // Check for superscript notation (², ³, etc.)
    if let Some((base, exp)) = parse_superscript(s) {
        let base_unit = lookup_base_unit(base)?;
        return Ok(base_unit.power(exp, 50));
    }

    // Simple unit
    lookup_base_unit(s)
}

/// Parse superscript exponents like m², m³
fn parse_superscript(s: &str) -> Option<(&str, i32)> {
    let superscripts = [
        ("⁰", 0), ("¹", 1), ("²", 2), ("³", 3), ("⁴", 4),
        ("⁵", 5), ("⁶", 6), ("⁷", 7), ("⁸", 8), ("⁹", 9),
        ("⁻", -1), // negative marker
    ];

    for (suffix, exp) in &superscripts {
        if s.ends_with(suffix) {
            let base = &s[..s.len() - suffix.len()];
            // Handle negative exponents like m⁻¹
            if *suffix == "⁻" && base.len() < s.len() {
                // Need to parse the next superscript digit
                continue;
            }
            return Some((base, *exp));
        }
    }

    // Check for compound superscripts like ⁻¹, ⁻²
    if s.contains('⁻') {
        if s.ends_with("⁻¹") {
            return Some((&s[..s.len() - "⁻¹".len()], -1));
        }
        if s.ends_with("⁻²") {
            return Some((&s[..s.len() - "⁻²".len()], -2));
        }
        if s.ends_with("⁻³") {
            return Some((&s[..s.len() - "⁻³".len()], -3));
        }
    }

    None
}

/// Look up a base unit by symbol or alias
fn lookup_base_unit(s: &str) -> Result<Unit, ConversionError> {
    let s = s.trim();

    if s == "1" || s.is_empty() {
        return Ok(Unit::new("1", "dimensionless", Dimension::DIMENSIONLESS, Number::from_i64(1), "dimensionless"));
    }

    UNITS.get(s)
        .cloned()
        .ok_or_else(|| ConversionError::UnknownUnit(s.to_string()))
}

/// Parse a conversion specification like "kg->lb" or "C->F"
pub fn parse_conversion(s: &str) -> Result<(Unit, Unit), ConversionError> {
    // Try different arrow formats
    let parts: Vec<&str> = if s.contains("->") {
        s.split("->").collect()
    } else if s.contains("→") {
        s.split("→").collect()
    } else if s.contains(" to ") {
        s.split(" to ").collect()
    } else if s.contains(" in ") {
        s.split(" in ").collect()
    } else {
        return Err(ConversionError::UnknownUnit(
            format!("invalid conversion format: {}, expected 'unit1->unit2'", s)
        ));
    };

    if parts.len() != 2 {
        return Err(ConversionError::UnknownUnit(
            format!("invalid conversion format: {}, expected 'unit1->unit2'", s)
        ));
    }

    let from_unit = parse_unit(parts[0])?;
    let to_unit = parse_unit(parts[1])?;

    Ok((from_unit, to_unit))
}

/// Parse a quantity string like "5 m" or "100 kg"
pub fn parse_quantity_string(s: &str) -> Result<(Number, Unit), ConversionError> {
    let s = s.trim();

    // Find where the number ends and unit begins
    let mut split_pos = 0;
    let mut found_digit = false;

    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
            found_digit = true;
            split_pos = i + c.len_utf8();
        } else if found_digit && c.is_whitespace() {
            // Skip whitespace after number
            split_pos = i;
            break;
        } else if found_digit {
            // Unit starts here
            split_pos = i;
            break;
        }
    }

    if !found_digit {
        return Err(ConversionError::UnknownUnit(
            format!("no number found in: {}", s)
        ));
    }

    let num_str = s[..split_pos].trim();
    let unit_str = s[split_pos..].trim();

    let value = Number::from_str(num_str)
        .map_err(|_| ConversionError::UnknownUnit(format!("invalid number: {}", num_str)))?;

    let unit = if unit_str.is_empty() {
        Unit::new("", "dimensionless", Dimension::DIMENSIONLESS, Number::from_i64(1), "dimensionless")
    } else {
        parse_unit(unit_str)?
    };

    Ok((value, unit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_unit() {
        let unit = parse_unit("m").unwrap();
        assert_eq!(unit.symbol, "m");
        assert_eq!(unit.dimension, Dimension::LENGTH);
    }

    #[test]
    fn test_parse_unit_with_power() {
        let unit = parse_unit("m^2").unwrap();
        assert_eq!(unit.dimension, Dimension::AREA);

        let unit = parse_unit("s^-1").unwrap();
        assert_eq!(unit.dimension, Dimension::FREQUENCY);
    }

    #[test]
    fn test_parse_quotient() {
        let unit = parse_unit("m/s").unwrap();
        assert_eq!(unit.dimension, Dimension::VELOCITY);
    }

    #[test]
    fn test_parse_product() {
        let unit = parse_unit("kg*m").unwrap();
        // kg*m has dimensions M*L
        let expected = Dimension::MASS.multiply(&Dimension::LENGTH);
        assert_eq!(unit.dimension, expected);
    }

    #[test]
    fn test_parse_complex() {
        // Force: kg*m/s^2
        let unit = parse_unit("kg*m/s^2").unwrap();
        assert_eq!(unit.dimension, Dimension::FORCE);
    }

    #[test]
    fn test_parse_conversion() {
        let (from, to) = parse_conversion("km->mi").unwrap();
        assert_eq!(from.symbol, "km");
        assert_eq!(to.symbol, "mi");
    }

    #[test]
    fn test_parse_conversion_arrow() {
        let (from, to) = parse_conversion("C→F").unwrap();
        // C and F are aliases for degC and degF
        assert_eq!(from.symbol, "degC");
        assert_eq!(to.symbol, "degF");
    }

    #[test]
    fn test_parse_quantity_string() {
        let (value, unit) = parse_quantity_string("5 m").unwrap();
        assert_eq!(value, Number::from_i64(5));
        assert_eq!(unit.symbol, "m");

        let (value, unit) = parse_quantity_string("100kg").unwrap();
        assert_eq!(value, Number::from_i64(100));
        assert_eq!(unit.symbol, "kg");

        let (value, unit) = parse_quantity_string("-3.14 rad").unwrap();
        let expected = Number::from_str("-3.14").unwrap();
        assert_eq!(value, expected);
        assert_eq!(unit.symbol, "rad");
    }

    #[test]
    fn test_alias_lookup() {
        // Test that aliases work
        let unit = parse_unit("meter").unwrap();
        assert_eq!(unit.symbol, "m");

        let unit = parse_unit("kilogram").unwrap();
        assert_eq!(unit.symbol, "kg");
    }

    #[test]
    fn test_unknown_unit() {
        let result = parse_unit("unknown_xyz");
        assert!(result.is_err());
    }
}
