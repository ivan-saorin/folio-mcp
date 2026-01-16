//! Dimensional analysis types
//!
//! Each physical quantity has dimensions represented as a 7-element vector:
//! [length, mass, time, current, temperature, amount, luminosity]

use std::fmt;
use serde::{Serialize, Deserialize};

/// Dimension indices for the 7 SI base quantities
pub const LENGTH: usize = 0;
pub const MASS: usize = 1;
pub const TIME: usize = 2;
pub const CURRENT: usize = 3;
pub const TEMPERATURE: usize = 4;
pub const AMOUNT: usize = 5;
pub const LUMINOSITY: usize = 6;

/// Represents the dimensions of a physical quantity
/// as exponents of the 7 SI base dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dimension {
    /// [length, mass, time, current, temperature, amount, luminosity]
    pub exponents: [i32; 7],
}

impl Dimension {
    /// Dimensionless quantity (all exponents zero)
    pub const DIMENSIONLESS: Dimension = Dimension { exponents: [0, 0, 0, 0, 0, 0, 0] };

    /// Length dimension [L]
    pub const LENGTH: Dimension = Dimension { exponents: [1, 0, 0, 0, 0, 0, 0] };

    /// Mass dimension [M]
    pub const MASS: Dimension = Dimension { exponents: [0, 1, 0, 0, 0, 0, 0] };

    /// Time dimension [T]
    pub const TIME: Dimension = Dimension { exponents: [0, 0, 1, 0, 0, 0, 0] };

    /// Electric current dimension [I]
    pub const CURRENT: Dimension = Dimension { exponents: [0, 0, 0, 1, 0, 0, 0] };

    /// Temperature dimension [Θ]
    pub const TEMPERATURE: Dimension = Dimension { exponents: [0, 0, 0, 0, 1, 0, 0] };

    /// Amount of substance dimension [N]
    pub const AMOUNT: Dimension = Dimension { exponents: [0, 0, 0, 0, 0, 1, 0] };

    /// Luminous intensity dimension [J]
    pub const LUMINOSITY: Dimension = Dimension { exponents: [0, 0, 0, 0, 0, 0, 1] };

    /// Velocity [L T^-1]
    pub const VELOCITY: Dimension = Dimension { exponents: [1, 0, -1, 0, 0, 0, 0] };

    /// Acceleration [L T^-2]
    pub const ACCELERATION: Dimension = Dimension { exponents: [1, 0, -2, 0, 0, 0, 0] };

    /// Force [M L T^-2]
    pub const FORCE: Dimension = Dimension { exponents: [1, 1, -2, 0, 0, 0, 0] };

    /// Energy [M L^2 T^-2]
    pub const ENERGY: Dimension = Dimension { exponents: [2, 1, -2, 0, 0, 0, 0] };

    /// Power [M L^2 T^-3]
    pub const POWER: Dimension = Dimension { exponents: [2, 1, -3, 0, 0, 0, 0] };

    /// Pressure [M L^-1 T^-2]
    pub const PRESSURE: Dimension = Dimension { exponents: [-1, 1, -2, 0, 0, 0, 0] };

    /// Area [L^2]
    pub const AREA: Dimension = Dimension { exponents: [2, 0, 0, 0, 0, 0, 0] };

    /// Volume [L^3]
    pub const VOLUME: Dimension = Dimension { exponents: [3, 0, 0, 0, 0, 0, 0] };

    /// Frequency [T^-1]
    pub const FREQUENCY: Dimension = Dimension { exponents: [0, 0, -1, 0, 0, 0, 0] };

    /// Electric charge [I T]
    pub const CHARGE: Dimension = Dimension { exponents: [0, 0, 1, 1, 0, 0, 0] };

    /// Voltage [M L^2 T^-3 I^-1]
    pub const VOLTAGE: Dimension = Dimension { exponents: [2, 1, -3, -1, 0, 0, 0] };

    /// Resistance [M L^2 T^-3 I^-2]
    pub const RESISTANCE: Dimension = Dimension { exponents: [2, 1, -3, -2, 0, 0, 0] };

    /// Create a new dimension from exponents
    pub fn new(exponents: [i32; 7]) -> Self {
        Dimension { exponents }
    }

    /// Check if this is a dimensionless quantity
    pub fn is_dimensionless(&self) -> bool {
        self.exponents.iter().all(|&e| e == 0)
    }

    /// Multiply dimensions (add exponents)
    pub fn multiply(&self, other: &Dimension) -> Dimension {
        let mut result = [0i32; 7];
        for i in 0..7 {
            result[i] = self.exponents[i] + other.exponents[i];
        }
        Dimension { exponents: result }
    }

    /// Divide dimensions (subtract exponents)
    pub fn divide(&self, other: &Dimension) -> Dimension {
        let mut result = [0i32; 7];
        for i in 0..7 {
            result[i] = self.exponents[i] - other.exponents[i];
        }
        Dimension { exponents: result }
    }

    /// Raise to integer power (multiply exponents)
    pub fn power(&self, exp: i32) -> Dimension {
        let mut result = [0i32; 7];
        for i in 0..7 {
            result[i] = self.exponents[i] * exp;
        }
        Dimension { exponents: result }
    }

    /// Invert dimensions (negate exponents)
    pub fn invert(&self) -> Dimension {
        self.power(-1)
    }

    /// Get the dimension name if it matches a common dimension
    pub fn name(&self) -> Option<&'static str> {
        match self.exponents {
            [0, 0, 0, 0, 0, 0, 0] => Some("dimensionless"),
            [1, 0, 0, 0, 0, 0, 0] => Some("length"),
            [0, 1, 0, 0, 0, 0, 0] => Some("mass"),
            [0, 0, 1, 0, 0, 0, 0] => Some("time"),
            [0, 0, 0, 1, 0, 0, 0] => Some("current"),
            [0, 0, 0, 0, 1, 0, 0] => Some("temperature"),
            [0, 0, 0, 0, 0, 1, 0] => Some("amount"),
            [0, 0, 0, 0, 0, 0, 1] => Some("luminosity"),
            [1, 0, -1, 0, 0, 0, 0] => Some("velocity"),
            [1, 0, -2, 0, 0, 0, 0] => Some("acceleration"),
            [1, 1, -2, 0, 0, 0, 0] => Some("force"),
            [2, 1, -2, 0, 0, 0, 0] => Some("energy"),
            [2, 1, -3, 0, 0, 0, 0] => Some("power"),
            [-1, 1, -2, 0, 0, 0, 0] => Some("pressure"),
            [2, 0, 0, 0, 0, 0, 0] => Some("area"),
            [3, 0, 0, 0, 0, 0, 0] => Some("volume"),
            [0, 0, -1, 0, 0, 0, 0] => Some("frequency"),
            _ => None,
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = ["L", "M", "T", "I", "Θ", "N", "J"];
        let mut parts = Vec::new();

        for (i, &exp) in self.exponents.iter().enumerate() {
            if exp != 0 {
                if exp == 1 {
                    parts.push(names[i].to_string());
                } else {
                    parts.push(format!("{}^{}", names[i], exp));
                }
            }
        }

        if parts.is_empty() {
            write!(f, "1")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::DIMENSIONLESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensionless() {
        assert!(Dimension::DIMENSIONLESS.is_dimensionless());
        assert!(!Dimension::LENGTH.is_dimensionless());
    }

    #[test]
    fn test_multiply() {
        let velocity = Dimension::LENGTH.divide(&Dimension::TIME);
        assert_eq!(velocity, Dimension::VELOCITY);
    }

    #[test]
    fn test_force() {
        // Force = Mass * Acceleration = M * L * T^-2
        let force = Dimension::MASS.multiply(&Dimension::ACCELERATION);
        assert_eq!(force, Dimension::FORCE);
    }

    #[test]
    fn test_power() {
        let area = Dimension::LENGTH.power(2);
        assert_eq!(area, Dimension::AREA);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Dimension::DIMENSIONLESS), "1");
        assert_eq!(format!("{}", Dimension::LENGTH), "L");
        assert_eq!(format!("{}", Dimension::VELOCITY), "L T^-1");
    }
}
