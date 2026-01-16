//! Arbitrary precision numbers using dashu
//!
//! Uses dashu-float (DBig) for arbitrary precision decimal arithmetic.
//! Native support for transcendentals (ln, exp, sqrt) without
//! the denominator explosion issues of rational arithmetic.

use dashu_float::DBig;
use dashu_float::ops::{SquareRoot, Abs};
use dashu_int::IBig;
use dashu_int::ops::BitTest;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use thiserror::Error;

/// Error type for number operations
#[derive(Debug, Clone, Error)]
pub enum NumberError {
    #[error("Invalid number format: {0}")]
    ParseError(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Overflow: result too large")]
    Overflow,
}

/// Default precision for calculations (decimal digits)
const DEFAULT_PRECISION: usize = 50;

/// Arbitrary precision decimal number
/// 
/// Built on dashu-float's DBig for efficient transcendental operations.
/// All operations return Results or new Numbers - never panic.
#[derive(Debug, Clone)]
pub struct Number {
    inner: DBig,
}

impl Number {
    // ========== Construction ==========

    /// Ensure a DBig has adequate precision for calculations
    fn with_work_precision(val: DBig) -> DBig {
        val.with_precision(DEFAULT_PRECISION).value()
    }

    /// Create from string representation
    /// Supports: "123", "3.14", "1/3", "1.5e10", "-42"
    pub fn from_str(s: &str) -> Result<Self, NumberError> {
        let s = s.trim();
        
        // Handle rational format "a/b"
        if s.contains('/') && !s.contains('.') && !s.contains('e') && !s.contains('E') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                let num_str = parts[0].trim();
                let den_str = parts[1].trim();
                
                let num: DBig = num_str.parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let den: DBig = den_str.parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                
                if den == DBig::ZERO {
                    return Err(NumberError::DivisionByZero);
                }
                
                let result = Self::with_work_precision(num) / Self::with_work_precision(den);
                return Ok(Self { inner: result });
            }
        }

        // Handle scientific notation with integer mantissa: "602214076e15"
        if (s.contains('e') || s.contains('E')) && !s.contains('.') {
            let s_lower = s.to_lowercase();
            let parts: Vec<&str> = s_lower.split('e').collect();
            if parts.len() == 2 {
                let mantissa: IBig = parts[0].parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let exp: i32 = parts[1].parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                
                // Use DBig::from_parts for exact scientific notation
                // significand * 10^exponent
                let result = DBig::from_parts(mantissa, exp as isize);
                return Ok(Self { inner: Self::with_work_precision(result) });
            }
        }

        // Standard decimal parsing
        let inner: DBig = s.parse()
            .map_err(|_| NumberError::ParseError(s.to_string()))?;
        
        Ok(Self { inner: Self::with_work_precision(inner) })
    }

    /// Create from i64 with working precision
    pub fn from_i64(n: i64) -> Self {
        Self { inner: Self::with_work_precision(DBig::from(n)) }
    }

    /// Create from ratio (exact division)
    pub fn from_ratio(num: i64, den: i64) -> Self {
        if den == 0 {
            return Self { inner: DBig::ZERO };
        }
        let n = Self::with_work_precision(DBig::from(num));
        let d = Self::with_work_precision(DBig::from(den));
        Self { inner: n / d }
    }

    /// Create from f64 (may lose precision for very large or very small values)
    pub fn from_f64(f: f64) -> Self {
        if f.is_nan() || f.is_infinite() {
            return Self { inner: DBig::ZERO };
        }
        // Use string conversion to preserve decimal precision
        let s = format!("{:.15}", f);
        Self::from_str(&s).unwrap_or(Self { inner: DBig::ZERO })
    }

    // ========== Predicates ==========

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.inner == DBig::ZERO
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.inner < DBig::ZERO
    }

    /// Check if value is an integer
    pub fn is_integer(&self) -> bool {
        let floor_val = self.inner.clone().floor();
        self.inner == floor_val
    }

    // ========== Basic Arithmetic ==========

    /// Addition
    pub fn add(&self, other: &Self) -> Self {
        Self { inner: &self.inner + &other.inner }
    }

    /// Subtraction
    pub fn sub(&self, other: &Self) -> Self {
        Self { inner: &self.inner - &other.inner }
    }

    /// Multiplication
    pub fn mul(&self, other: &Self) -> Self {
        Self { inner: &self.inner * &other.inner }
    }

    /// Safe division (returns Result, never panics)
    pub fn checked_div(&self, other: &Self) -> Result<Self, NumberError> {
        if other.is_zero() {
            Err(NumberError::DivisionByZero)
        } else {
            Ok(Self { inner: &self.inner / &other.inner })
        }
    }

    /// Integer power (exact)
    pub fn pow(&self, exp: i32) -> Self {
        if exp == 0 {
            return Self::from_i64(1);
        }
        
        let abs_exp = exp.unsigned_abs();
        let mut result = Self::from_i64(1);
        
        // Simple repeated multiplication
        for _ in 0..abs_exp {
            result = result.mul(self);
        }
        
        if exp < 0 {
            Self::from_i64(1).checked_div(&result).unwrap_or(Self::from_i64(0))
        } else {
            result
        }
    }

    /// Real-valued power: x^y = exp(y * ln(x))
    pub fn pow_real(&self, exp: &Self, precision: u32) -> Self {
        if exp.is_zero() {
            return Self::from_i64(1);
        }
        if self.is_zero() {
            return Self::from_i64(0);
        }

        // If exponent is a small integer, use exact power
        if exp.is_integer() {
            if let Some(e) = exp.to_i64() {
                if e.abs() <= i32::MAX as i64 {
                    return self.pow(e as i32);
                }
            }
        }

        // For x^y where x > 0: x^y = exp(y * ln(x))
        if self.is_negative() {
            return Self::from_i64(0);
        }

        let ln_x = self.inner.clone().with_precision(precision as usize).value().ln();
        let product = &ln_x * &exp.inner;
        Self { inner: product.exp() }
    }

    // ========== Transcendental Functions ==========

    /// Square root
    pub fn sqrt(&self, precision: u32) -> Result<Self, NumberError> {
        if self.is_negative() {
            return Err(NumberError::DomainError(
                "square root of negative number".to_string()
            ));
        }
        if self.is_zero() {
            return Ok(Self::from_i64(0));
        }

        let val = self.inner.clone().with_precision(precision as usize).value();
        Ok(Self { inner: val.sqrt() })
    }

    /// Natural logarithm
    pub fn ln(&self, precision: u32) -> Result<Self, NumberError> {
        if self.inner <= DBig::ZERO {
            return Err(NumberError::DomainError(
                "logarithm of non-positive number".to_string()
            ));
        }

        let val = self.inner.clone().with_precision(precision as usize).value();
        Ok(Self { inner: val.ln() })
    }

    /// Exponential function (e^x)
    pub fn exp(&self, precision: u32) -> Self {
        let val = self.inner.clone().with_precision(precision as usize).value();
        Self { inner: val.exp() }
    }

    /// Sine function (Taylor series)
    pub fn sin(&self, precision: u32) -> Self {
        let x = self.inner.clone().with_precision(precision as usize).value();
        let x_squared = &x * &x;
        
        let mut sum = x.clone();
        let mut term = x.clone();
        
        let iterations = (precision / 3).max(12).min(50) as i64;
        for k in 1..iterations {
            let denom = DBig::from((2 * k) * (2 * k + 1));
            term = -&term * &x_squared / denom;
            sum = &sum + &term;
        }
        
        Self { inner: sum }
    }

    /// Cosine function (Taylor series)
    pub fn cos(&self, precision: u32) -> Self {
        let x = self.inner.clone().with_precision(precision as usize).value();
        let x_squared = &x * &x;
        
        let one = DBig::ONE.with_precision(precision as usize).value();
        let mut sum = one.clone();
        let mut term = one;
        
        let iterations = (precision / 3).max(12).min(50) as i64;
        for k in 1..iterations {
            let denom = DBig::from((2 * k - 1) * (2 * k));
            term = -&term * &x_squared / denom;
            sum = &sum + &term;
        }
        
        Self { inner: sum }
    }

    /// Tangent function (sin/cos)
    pub fn tan(&self, precision: u32) -> Result<Self, NumberError> {
        let cos_x = self.cos(precision);
        if cos_x.is_zero() {
            return Err(NumberError::DomainError(
                "tan undefined at odd multiples of π/2".to_string()
            ));
        }
        let sin_x = self.sin(precision);
        sin_x.checked_div(&cos_x)
    }

    // ========== Mathematical Constants ==========

    /// Golden ratio φ = (1 + √5) / 2
    pub fn phi(precision: u32) -> Self {
        let five = Self::from_i64(5);
        let sqrt5 = five.sqrt(precision + 10).unwrap_or(Self::from_i64(2));
        let one = Self::from_i64(1);
        let two = Self::from_i64(2);
        one.add(&sqrt5).checked_div(&two).unwrap_or(Self::from_ratio(161803, 100000))
    }

    /// Pi - from high-precision string constant
    pub fn pi(precision: u32) -> Self {
        const PI_STR: &str = "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458700660631558817488152092096282925409171536436789259036001133053054882046652138414695194151160943305727036575959195309218611738193261179310511854807446237996274956735188575272489122793818301194912";
        
        let end_pos = (precision as usize + 2).min(PI_STR.len());
        Self::from_str(&PI_STR[..end_pos])
            .unwrap_or(Self::from_ratio(355, 113))
    }

    /// Euler's number e
    pub fn e(precision: u32) -> Self {
        Self::from_i64(1).exp(precision)
    }

    // ========== Other Operations ==========

    /// Absolute value
    pub fn abs(&self) -> Self {
        Self { inner: Abs::abs(self.inner.clone()) }
    }

    /// Floor - largest integer <= x
    pub fn floor(&self) -> Self {
        Self { inner: self.inner.clone().floor() }
    }

    /// Ceiling - smallest integer >= x
    pub fn ceil(&self) -> Self {
        Self { inner: self.inner.clone().ceil() }
    }

    /// Try to convert to i64
    pub fn to_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        
        // DBig stores as significand * 10^exponent
        let (significand, exponent) = self.inner.clone().into_repr().into_parts();
        
        // Try to get i64 from significand
        let sig_i64: i64 = significand.try_into().ok()?;
        
        if exponent == 0 {
            Some(sig_i64)
        } else if exponent > 0 && exponent <= 18 {
            sig_i64.checked_mul(10_i64.checked_pow(exponent as u32)?)
        } else if exponent < 0 && exponent >= -18 {
            let divisor = 10_i64.checked_pow((-exponent) as u32)?;
            if sig_i64 % divisor == 0 {
                Some(sig_i64 / divisor)
            } else {
                None
            }
        } else {
            // Fall back to f64 conversion
            self.to_f64().and_then(|f| {
                if f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    Some(f as i64)
                } else {
                    None
                }
            })
        }
    }

    // ========== Display ==========

    /// Render as decimal string with specified decimal places
    pub fn as_decimal(&self, places: u32) -> String {
        if let Some(f) = self.to_f64() {
            // Handle very small non-zero numbers
            if f != 0.0 && f.abs() < 1e-6 {
                let log10 = f.abs().log10().floor() as i32;
                let sig_places = ((-log10) + 2) as usize;
                return format!("{:.prec$}", f, prec = sig_places);
            }

            if places == 0 {
                format!("{:.0}", f)
            } else {
                format!("{:.prec$}", f, prec = places as usize)
            }
        } else {
            format!("{}", self.inner)
        }
    }

    /// Render with N significant figures
    pub fn as_sigfigs(&self, sigfigs: u32) -> String {
        if let Some(f) = self.to_f64() {
            if f == 0.0 {
                return "0".to_string();
            }

            let sigfigs = sigfigs.max(1) as usize;
            let exp = f.abs().log10().floor() as i32;

            if exp >= -3 && exp <= 4 {
                let decimal_places = if exp >= 0 {
                    (sigfigs as i32 - exp - 1).max(0) as usize
                } else {
                    sigfigs + (-exp - 1) as usize
                };
                format!("{:.prec$}", f, prec = decimal_places)
            } else {
                let mantissa = f / 10_f64.powi(exp);
                let decimal_places = (sigfigs - 1).max(0);
                format!("{:.prec$}e{}", mantissa, exp, prec = decimal_places)
            }
        } else {
            format!("{}", self.inner)
        }
    }

    /// Convert to f64 (may lose precision)
    pub fn to_f64(&self) -> Option<f64> {
        // Get the representation: significand * 10^exponent
        let (significand, exponent) = self.inner.clone().into_repr().into_parts();
        
        // Convert significand to f64
        // For large significands, we need to be careful
        let sig_f64: f64 = if significand.bit_len() <= 53 {
            // Safe direct conversion
            match TryInto::<i64>::try_into(significand.clone()) {
                Ok(i) => i as f64,
                Err(_) => {
                    // Try as u64 then negate if needed
                    let is_neg = significand < IBig::ZERO;
                    let abs_sig = if is_neg { -significand.clone() } else { significand.clone() };
                    match TryInto::<u64>::try_into(abs_sig) {
                        Ok(u) => if is_neg { -(u as f64) } else { u as f64 },
                        Err(_) => return None,
                    }
                }
            }
        } else {
            // Significand too large - need to scale down
            // Shift right to fit in 53 bits, adjusting exponent
            let extra_bits = significand.bit_len() - 53;
            let shifted = &significand >> extra_bits;
            let shifted_i64: i64 = shifted.try_into().ok()?;
            let base_f64 = shifted_i64 as f64;
            // Account for the bits we shifted out
            base_f64 * 2_f64.powi(extra_bits as i32)
        };
        
        // Apply the decimal exponent
        let result = if exponent == 0 {
            sig_f64
        } else if exponent > 0 && exponent <= 308 {
            sig_f64 * 10_f64.powi(exponent as i32)
        } else if exponent < 0 && exponent >= -308 {
            sig_f64 / 10_f64.powi((-exponent) as i32)
        } else {
            return None; // Exponent out of f64 range
        };
        
        if result.is_finite() {
            Some(result)
        } else {
            None
        }
    }
}

// ========== Trait Implementations ==========

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_decimal(10))
    }
}

impl Serialize for Number {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // DBig implements PartialOrd, use it and treat None as Equal
        self.inner.partial_cmp(&other.inner).unwrap_or(std::cmp::Ordering::Equal)
    }
}
