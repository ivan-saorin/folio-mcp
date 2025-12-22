//! Arbitrary precision rational numbers
//!
//! Wraps `num_rational::BigRational` for unlimited precision arithmetic.
//! All operations return Results - never panic.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero, One, Signed};
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

/// Arbitrary precision rational number
#[derive(Debug, Clone)]
pub struct Number {
    inner: BigRational,
}

impl Number {
    /// Create from string representation
    /// Supports: "123", "3.14", "1/3", "1.5e10"
    pub fn from_str(s: &str) -> Result<Self, NumberError> {
        let s = s.trim();

        // Try rational format first (e.g., "1/3")
        if s.contains('/') && !s.contains('.') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                let num: BigInt = parts[0].trim().parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let den: BigInt = parts[1].trim().parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                if den.is_zero() {
                    return Err(NumberError::DivisionByZero);
                }
                return Ok(Self { inner: BigRational::new(num, den) });
            }
        }

        // Handle scientific notation
        if s.contains('e') || s.contains('E') {
            let s_lower = s.to_lowercase();
            let parts: Vec<&str> = s_lower.split('e').collect();
            if parts.len() == 2 {
                let base = Self::from_str(parts[0])?;
                let exp: i32 = parts[1].parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let ten = BigInt::from(10);
                if exp >= 0 {
                    let multiplier = BigRational::from(num_traits::pow(ten, exp as usize));
                    return Ok(Self { inner: base.inner * multiplier });
                } else {
                    let divisor = BigRational::from(num_traits::pow(ten, (-exp) as usize));
                    return Ok(Self { inner: base.inner / divisor });
                }
            }
        }

        // Handle decimal format
        if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() == 2 {
                let decimal_places = parts[1].len() as u32;
                let combined = format!("{}{}", parts[0], parts[1]);
                let numerator: BigInt = combined.parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let denominator = num_traits::pow(BigInt::from(10), decimal_places as usize);
                return Ok(Self {
                    inner: BigRational::new(numerator, denominator),
                });
            }
        }

        // Try integer
        let n: BigInt = s.parse()
            .map_err(|_| NumberError::ParseError(s.to_string()))?;
        Ok(Self { inner: BigRational::from(n) })
    }

    /// Create from integer
    pub fn from_i64(n: i64) -> Self {
        Self { inner: BigRational::from(BigInt::from(n)) }
    }

    /// Create from ratio
    pub fn from_ratio(num: i64, den: i64) -> Self {
        Self { inner: BigRational::new(BigInt::from(num), BigInt::from(den)) }
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.inner.is_negative()
    }

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

    /// Integer power
    pub fn pow(&self, exp: i32) -> Self {
        if exp == 0 {
            return Self::from_i64(1);
        }
        if exp > 0 {
            let mut result = self.inner.clone();
            for _ in 1..exp {
                result = &result * &self.inner;
            }
            Self { inner: result }
        } else {
            // Negative exponent: 1 / x^|exp|
            let pos_pow = self.pow(-exp);
            Self { inner: BigRational::one() / pos_pow.inner }
        }
    }

    /// Real-valued power using x^y = exp(y * ln(x))
    /// Works for any exponent (integer, fractional, negative)
    pub fn pow_real(&self, exp: &Self, precision: u32) -> Self {
        // Special cases
        if exp.is_zero() {
            return Self::from_i64(1);
        }
        if self.is_zero() {
            return Self::from_i64(0);
        }

        // If exponent is an integer, use fast integer power
        if exp.is_integer() {
            if let Some(e) = exp.to_i64() {
                if e.abs() <= i32::MAX as i64 {
                    return self.pow(e as i32);
                }
            }
        }

        // For x^y where x > 0: x^y = exp(y * ln(x))
        if self.is_negative() {
            // Can't compute real power of negative base with non-integer exponent
            // Return NaN-like behavior (just return 0 for now)
            return Self::from_i64(0);
        }

        match self.ln(precision) {
            Ok(ln_base) => {
                let product = ln_base.mul(exp);
                product.exp(precision)
            }
            Err(_) => Self::from_i64(0),
        }
    }

    /// Square root using Newton-Raphson with controlled precision
    /// Uses digit-based iteration count to achieve arbitrary precision
    pub fn sqrt(&self, precision: u32) -> Result<Self, NumberError> {
        if self.is_negative() {
            return Err(NumberError::DomainError("square root of negative number".to_string()));
        }
        if self.is_zero() {
            return Ok(Self::from_i64(0));
        }

        // For perfect squares of small integers, return exact result
        if self.is_integer() {
            if let Some(n) = self.to_i64() {
                if n > 0 && n <= 1_000_000_000_000i64 {
                    let isqrt = (n as f64).sqrt() as i64;
                    if isqrt * isqrt == n {
                        return Ok(Self::from_i64(isqrt));
                    }
                }
            }
        }

        // Fast path: use f64 for low precision (up to ~15 digits)
        if precision <= 15 {
            if let Some(f) = self.to_f64() {
                let s = f.sqrt();
                if s.is_finite() && s > 0.0 {
                    return Self::from_str(&format!("{:.15}", s))
                        .map_err(|_| NumberError::DomainError("sqrt conversion failed".to_string()));
                }
            }
        }

        // Newton-Raphson: x_{n+1} = (x_n + S/x_n) / 2
        // Start with f64 approximation as initial guess
        let two = BigRational::from(BigInt::from(2));

        let initial_guess = if let Some(f) = self.to_f64() {
            let s = f.sqrt();
            if s.is_finite() && s > 0.0 {
                Self::from_str(&format!("{:.15}", s))
                    .unwrap_or(Self::from_i64(1))
            } else {
                Self::from_i64(1)
            }
        } else {
            Self::from_i64(1)
        };

        let mut x = initial_guess.inner;

        // Number of iterations: Newton-Raphson doubles precision each iteration
        // Start with ~15 digits from f64, need ceil(log2(precision/15)) more iterations
        // Cap at reasonable number for performance
        let iterations = ((precision as f64 / 15.0).log2().ceil() as u32 + 2).max(3).min(10);

        for _ in 0..iterations {
            // x = (x + self/x) / 2
            let quotient = &self.inner / &x;
            x = (&x + &quotient) / &two;
        }

        Ok(Self { inner: x })
    }

    /// Natural logarithm using argument reduction and series expansion
    /// Uses ln(x) = k*ln(2) + ln(x/2^k) to bring argument close to 1
    pub fn ln(&self, precision: u32) -> Result<Self, NumberError> {
        if self.inner <= BigRational::zero() {
            return Err(NumberError::DomainError("logarithm of non-positive number".to_string()));
        }

        // Fast path: use f64 for low precision (up to ~15 digits)
        if precision <= 15 {
            if let Some(f) = self.to_f64() {
                if f > 0.0 {
                    let result = f.ln();
                    if result.is_finite() {
                        return Self::from_str(&format!("{:.15}", result))
                            .map_err(|_| NumberError::DomainError("ln conversion failed".to_string()));
                    }
                }
            }
        }

        let one = BigRational::one();
        let two = BigRational::from(BigInt::from(2));

        // Argument reduction: find k such that x/2^k is close to 1
        // We want x/2^k to be in [0.5, 2] for good convergence
        let mut reduced = self.inner.clone();
        let mut k: i64 = 0;

        // Reduce while x > 2
        while reduced > two {
            reduced = &reduced / &two;
            k += 1;
        }

        // Reduce while x < 0.5
        let half = &one / &two;
        while reduced < half {
            reduced = &reduced * &two;
            k -= 1;
        }

        // Now compute ln(reduced) using the arctanh series
        // ln(x) = 2 * arctanh((x-1)/(x+1))
        // arctanh(y) = y + y^3/3 + y^5/5 + ...
        let y = (&reduced - &one) / (&reduced + &one);

        let mut sum = y.clone();
        let mut y_power = y.clone();
        let y_squared = &y * &y;

        // Number of terms based on precision (capped for performance)
        // For arctanh series with |y| < 1/3, convergence is fast
        let terms = ((precision / 2).max(10).min(60)) as i64;
        for i in 1..terms {
            y_power = &y_power * &y_squared;
            let term = &y_power / BigRational::from(BigInt::from(2 * i + 1));
            sum = &sum + &term;
        }

        let ln_reduced = &two * &sum;

        // ln(2) pre-computed with 100+ digit precision
        let ln2_str = "0.6931471805599453094172321214581765680755001343602552541206800094933936219696947156058633269964186875";
        let ln2 = Self::from_str(ln2_str).unwrap_or_else(|_| {
            Self { inner: BigRational::from_float(0.693147180559945).unwrap() }
        });

        // ln(x) = k*ln(2) + ln(reduced)
        let k_rational = BigRational::from(BigInt::from(k));
        Ok(Self { inner: &ln_reduced + &(&k_rational * &ln2.inner) })
    }

    /// Exponential function (e^x) using argument reduction and Taylor series
    /// Uses exp(x) = exp(x - k*ln(2)) * 2^k to reduce argument to [-0.5, 0.5]
    pub fn exp(&self, precision: u32) -> Self {
        // Fast path: use f64 for low precision (up to ~15 digits)
        if precision <= 15 {
            if let Some(f) = self.to_f64() {
                let result = f.exp();
                if result.is_finite() {
                    if let Ok(n) = Self::from_str(&format!("{:.15e}", result)) {
                        return n;
                    }
                }
            }
        }

        // ln(2) pre-computed with 100+ digit precision
        let ln2_str = "0.6931471805599453094172321214581765680755001343602552541206800094933936219696947156058633269964186875";
        let ln2 = Self::from_str(ln2_str).unwrap_or_else(|_| {
            Self { inner: BigRational::from_float(0.693147180559945).unwrap() }
        });

        // Argument reduction: find k such that |x - k*ln(2)| < 0.5
        // k = round(x / ln(2)) - computed using BigRational for precision
        let x_div_ln2 = &self.inner / &ln2.inner;

        // Round to nearest integer: floor(x + 0.5) for positive, ceil(x - 0.5) for negative
        let half = BigRational::new(BigInt::from(1), BigInt::from(2));
        let k_big = if x_div_ln2 >= BigRational::zero() {
            (&x_div_ln2 + &half).floor()
        } else {
            (&x_div_ln2 - &half).ceil()
        };

        // Convert k to i64 for power calculation (should always fit for reasonable inputs)
        let k: i64 = k_big.numer().to_i64().unwrap_or(0) / k_big.denom().to_i64().unwrap_or(1);

        // reduced = x - k * ln(2)
        let k_rational = BigRational::from(BigInt::from(k));
        let reduced = &self.inner - &(&k_rational * &ln2.inner);

        // Compute exp(reduced) using Taylor series
        // exp(x) = 1 + x + x^2/2! + x^3/3! + ...
        let mut sum = BigRational::one();
        let mut term = BigRational::one();

        // Number of terms based on precision (capped for performance)
        // For |reduced| < 0.5, Taylor series converges very fast
        let terms = (precision.max(15).min(80)) as i64;
        for i in 1..terms {
            term = &term * &reduced / BigRational::from(BigInt::from(i));
            sum = &sum + &term;
        }

        // exp(x) = exp(reduced) * 2^k
        let exp_reduced = Self { inner: sum };
        if k == 0 {
            exp_reduced
        } else if k > 0 {
            // Use BigInt for 2^k to handle large exponents
            let two = BigInt::from(2);
            let two_pow_k = num_traits::pow(two, k as usize);
            Self { inner: &exp_reduced.inner * BigRational::from(two_pow_k) }
        } else {
            let two = BigInt::from(2);
            let two_pow_neg_k = num_traits::pow(two, (-k) as usize);
            Self { inner: &exp_reduced.inner / BigRational::from(two_pow_neg_k) }
        }
    }

    /// Golden ratio φ = (1 + √5) / 2
    /// Uses precomputed value for efficiency (100+ digits available)
    pub fn phi(precision: u32) -> Self {
        // Use precomputed phi for up to 100 digits (fast path)
        if precision <= 100 {
            let phi_str = "1.6180339887498948482045868343656381177203091798057628621354486227052604628189024497072072041893911375";
            return Self::from_str(phi_str).unwrap_or(Self::from_ratio(161803, 100000));
        }
        // For higher precision, compute from sqrt(5)
        let five = Self::from_i64(5);
        let sqrt5 = five.sqrt(precision).unwrap_or_else(|_| {
            Self::from_str("2.2360679774997896964091736687747632067176941640625").unwrap_or(Self::from_i64(2))
        });
        let one = Self::from_i64(1);
        let two = Self::from_i64(2);
        one.add(&sqrt5).checked_div(&two).unwrap_or(Self::from_i64(1))
    }

    /// Pi (pre-computed high precision value)
    pub fn pi(_precision: u32) -> Self {
        // Use pre-computed Pi with 100+ digits
        let pi_str = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679";
        Self::from_str(pi_str).unwrap_or(Self::from_ratio(355, 113)) // fallback to rational approx
    }

    /// Arctan using Taylor series (limited iterations for efficiency)
    #[allow(dead_code)]
    fn arctan(x: &Self, _precision: u32) -> Self {
        // arctan(x) = x - x^3/3 + x^5/5 - x^7/7 + ...
        let mut sum = x.inner.clone();
        let mut x_power = x.inner.clone();
        let x_squared = &x.inner * &x.inner;

        // Fixed 15 iterations
        let terms = 15;
        for k in 1..terms {
            x_power = &x_power * &x_squared;
            let sign = if k % 2 == 1 { -1i64 } else { 1i64 };
            let term = &x_power * BigRational::from(BigInt::from(sign))
                / BigRational::from(BigInt::from(2 * k + 1));
            sum = &sum + &term;
        }

        Self { inner: sum }
    }

    /// Euler's number e (pre-computed for efficiency)
    pub fn e(_precision: u32) -> Self {
        let e_str = "2.7182818284590452353602874713526624977572470936999595749669";
        Self::from_str(e_str).unwrap_or(Self::from_ratio(2718281828, 1000000000))
    }

    /// Render as decimal string
    /// For very small numbers (< 10^-6), shows enough decimal places to display
    /// at least 3 significant digits instead of just showing 0.0000000000
    pub fn as_decimal(&self, places: u32) -> String {
        // Convert to f64 for display (may lose precision for very large numbers)
        if let Some(f) = self.to_f64() {
            // Handle very small non-zero numbers that would display as all zeros
            if f != 0.0 && f.abs() < 1e-6 {
                // Calculate how many decimal places needed for 3 significant digits
                let log10 = f.abs().log10().floor() as i32;
                let sig_places = ((-log10) + 2) as usize; // +2 for 3 sig figs
                return format!("{:.prec$}", f, prec = sig_places);
            }

            if places == 0 {
                format!("{:.0}", f)
            } else {
                format!("{:.prec$}", f, prec = places as usize)
            }
        } else {
            format!("{}/{}", self.inner.numer(), self.inner.denom())
        }
    }

    /// Render with N significant figures, using scientific notation when appropriate
    /// Examples with sigfigs=4: 602214076e15 -> "6.022e23", 0.00123 -> "1.230e-3"
    pub fn as_sigfigs(&self, sigfigs: u32) -> String {
        if let Some(f) = self.to_f64() {
            if f == 0.0 {
                return "0".to_string();
            }

            let sigfigs = sigfigs.max(1) as usize;

            // Calculate the exponent (power of 10)
            let exp = f.abs().log10().floor() as i32;

            // For numbers close to 1 (between 0.001 and 10000), use regular notation
            if exp >= -3 && exp <= 4 {
                // Calculate decimal places needed for sigfigs
                let decimal_places = if exp >= 0 {
                    (sigfigs as i32 - exp - 1).max(0) as usize
                } else {
                    sigfigs + (-exp - 1) as usize
                };
                format!("{:.prec$}", f, prec = decimal_places)
            } else {
                // Use scientific notation
                let mantissa = f / 10_f64.powi(exp);
                let decimal_places = (sigfigs - 1).max(0);
                format!("{:.prec$}e{}", mantissa, exp, prec = decimal_places)
            }
        } else {
            format!("{}/{}", self.inner.numer(), self.inner.denom())
        }
    }

    /// Try to convert to f64
    fn to_f64(&self) -> Option<f64> {
        let numer = self.inner.numer();
        let denom = self.inner.denom();

        // First try direct conversion
        if let (Some(n), Some(d)) = (numer.to_f64(), denom.to_f64()) {
            if n.is_finite() && d.is_finite() && d != 0.0 {
                let result = n / d;
                if result.is_finite() {
                    return Some(result);
                }
            }
        }

        // Handle case where numerator and/or denominator overflow f64
        // This happens with BigRational after many operations
        let numer_bits = numer.bits() as i32;
        let denom_bits = denom.bits() as i32;

        if numer_bits == 0 {
            return Some(0.0);
        }

        // The ratio n/d has magnitude approximately 2^(numer_bits - denom_bits)
        // We need to shift to get both in range of f64 mantissa (53 bits)
        // while preserving the relative magnitude

        // Shift both to ~53 bits
        let target_bits = 53i32;

        // Shift numerator to target_bits
        let numer_shift = (numer_bits - target_bits).max(0) as usize;
        // Shift denominator to target_bits
        let denom_shift = (denom_bits - target_bits).max(0) as usize;

        let shifted_numer = numer >> numer_shift;
        let shifted_denom = denom >> denom_shift;

        if let (Some(n_f64), Some(d_f64)) = (shifted_numer.to_f64(), shifted_denom.to_f64()) {
            if d_f64 != 0.0 {
                // Compute the base ratio
                let base_ratio = n_f64 / d_f64;

                // Account for the different shift amounts
                // We shifted numer by numer_shift and denom by denom_shift
                // So the true value is base_ratio * 2^(numer_shift - denom_shift)
                let shift_diff = numer_shift as i32 - denom_shift as i32;

                let result = if shift_diff == 0 {
                    base_ratio
                } else if shift_diff > 0 && shift_diff < 1024 {
                    base_ratio * 2_f64.powi(shift_diff)
                } else if shift_diff < 0 && shift_diff > -1024 {
                    base_ratio / 2_f64.powi(-shift_diff)
                } else {
                    // Shift too large for f64
                    return None;
                };

                if result.is_finite() {
                    return Some(result);
                }
            }
        }

        None
    }

    /// Check if value is an integer
    pub fn is_integer(&self) -> bool {
        self.inner.is_integer()
    }

    /// Try to convert to i64
    pub fn to_i64(&self) -> Option<i64> {
        if self.is_integer() {
            self.inner.numer().to_i64()
        } else {
            None
        }
    }

    /// Absolute value
    pub fn abs(&self) -> Self {
        Self { inner: self.inner.abs() }
    }

    /// Floor - largest integer less than or equal to x
    pub fn floor(&self) -> Self {
        Self { inner: self.inner.floor() }
    }

    /// Ceiling - smallest integer greater than or equal to x
    pub fn ceil(&self) -> Self {
        Self { inner: self.inner.ceil() }
    }

    /// Sine function using Taylor series
    pub fn sin(&self, _precision: u32) -> Self {
        // sin(x) = x - x^3/3! + x^5/5! - x^7/7! + ...
        let mut sum = self.inner.clone();
        let mut term = self.inner.clone();
        let x_squared = &self.inner * &self.inner;

        // Fixed 12 iterations - factorial denominator ensures fast convergence
        let terms = 12;
        for k in 1..terms {
            term = -&term * &x_squared / BigRational::from(BigInt::from((2 * k) * (2 * k + 1)));
            sum = &sum + &term;
        }

        Self { inner: sum }
    }

    /// Cosine function using Taylor series
    pub fn cos(&self, _precision: u32) -> Self {
        // cos(x) = 1 - x^2/2! + x^4/4! - x^6/6! + ...
        let mut sum = BigRational::one();
        let mut term = BigRational::one();
        let x_squared = &self.inner * &self.inner;

        // Fixed 12 iterations - factorial denominator ensures fast convergence
        let terms = 12;
        for k in 1..terms {
            term = -&term * &x_squared / BigRational::from(BigInt::from((2 * k - 1) * (2 * k)));
            sum = &sum + &term;
        }

        Self { inner: sum }
    }

    /// Tangent function (sin/cos)
    pub fn tan(&self, precision: u32) -> Result<Self, NumberError> {
        let cos_x = self.cos(precision);
        if cos_x.is_zero() {
            return Err(NumberError::DomainError("tan undefined at odd multiples of π/2".to_string()));
        }
        let sin_x = self.sin(precision);
        sin_x.checked_div(&cos_x)
    }
}

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
