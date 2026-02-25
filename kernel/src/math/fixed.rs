//! `Fixed(u128)` — The constitutional fixed-point type.
//!
//! All Accountability magnitudes in the kernel are represented as Fixed values.
//! SCALE = 1_000_000_000_000 (10^12).
//!
//! INVARIANTS:
//! - The inner u128 value is PRIVATE. No consensus code touches `.0` directly.
//! - Every multiplication of two Fixed values calls `mul_scaled`, which divides
//!   by SCALE internally. Never multiply two Fixed values without that reduction.
//! - Every operation that can fail returns Result<Fixed, TransitionError>.
//! - Division by zero pre-checks the denominator and returns DivisionByZero,
//!   never a WASM trap.

use crate::TransitionError;

/// The scaling factor. 1.0 accountability unit = Fixed(1_000_000_000_000).
pub const SCALE: u128 = 1_000_000_000_000;

/// Maximum safe raw value before a decay multiplication (balance * decay_factor)
/// would overflow u128. Derived as: u128::MAX / SCALE.
/// Any Fixed value whose inner u128 exceeds this should be considered a protocol
/// invariant violation — individual balances must never reach this ceiling.
pub const MAX_SAFE_BALANCE_RAW: u128 = u128::MAX / SCALE;

/// The constitutional fixed-point type.
/// The inner value is private — enforced by the Rust module system.
/// Consensus code imports `Fixed` but cannot access `.0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Fixed(u128);

impl Fixed {
    /// Construct a Fixed from a pre-scaled raw u128.
    /// Returns an error if the raw value exceeds MAX_SAFE_BALANCE_RAW,
    /// as such values would overflow during a subsequent decay multiplication.
    pub fn from_raw(raw: u128) -> Result<Self, TransitionError> {
        if raw > MAX_SAFE_BALANCE_RAW {
            Err(TransitionError::MathOverflow)
        } else {
            Ok(Fixed(raw))
        }
    }

    /// Construct a Fixed from a whole-unit count (e.g., 5 = 5.0 accountability units).
    /// Equivalent to `Fixed::from_raw(whole_units * SCALE)`.
    pub fn from_units(whole_units: u128) -> Result<Self, TransitionError> {
        let raw = whole_units.checked_mul(SCALE).ok_or(TransitionError::MathOverflow)?;
        Self::from_raw(raw)
    }

    /// Parse a Fixed from a canonical numeric string (`^(0|[1-9][0-9]*)$`).
    /// The string represents the raw inner value (already scaled).
    /// Returns InvalidSerialization if the string violates the grammar.
    pub fn from_canonical_str(s: &str) -> Result<Self, TransitionError> {
        // Validate grammar: must be "0" or a non-zero-leading decimal digit string.
        let valid = s == "0" || (
            !s.is_empty()
            && s.as_bytes()[0] != b'0'
            && s.bytes().all(|b| b.is_ascii_digit())
        );
        if !valid {
            return Err(TransitionError::InvalidSerialization);
        }
        // Parse the validated string as u128.
        let raw = s.parse::<u128>().map_err(|_| TransitionError::MathOverflow)?;
        Self::from_raw(raw)
    }

    /// Returns the inner raw u128 value.
    /// ONLY for use inside the `math` module and test harnesses.
    /// Consensus code outside this module cannot call this.
    pub(crate) fn raw(self) -> u128 {
        self.0
    }

    /// Multiply two Fixed values, dividing by SCALE to keep the result scaled.
    /// Formula: (self.0 * other.0) / SCALE
    /// Uses checked_mul before the division to catch overflow before it occurs.
    pub fn mul_scaled(self, other: Fixed) -> Result<Fixed, TransitionError> {
        let product = self.0.checked_mul(other.0).ok_or(TransitionError::MathOverflow)?;
        let result = product / SCALE; // Integer division: truncation = floor (for unsigned)
        Self::from_raw(result)
    }

    /// Divide self by other, scaling correctly: (self.0 * SCALE) / other.0
    /// Pre-checks the denominator for zero before any division attempt.
    pub fn div_scaled(self, other: Fixed) -> Result<Fixed, TransitionError> {
        if other.0 == 0 {
            return Err(TransitionError::DivisionByZero);
        }
        let numerator = self.0.checked_mul(SCALE).ok_or(TransitionError::MathOverflow)?;
        let result = numerator / other.0; // Truncation = floor
        Self::from_raw(result)
    }

    /// Add two Fixed values. Returns overflow error if result exceeds MAX_SAFE_BALANCE_RAW.
    pub fn checked_add(self, other: Fixed) -> Result<Fixed, TransitionError> {
        let sum = self.0.checked_add(other.0).ok_or(TransitionError::MathOverflow)?;
        Self::from_raw(sum)
    }

    /// Subtract other from self. Returns overflow error if other > self.
    /// For slashing (which must clamp to zero), use `saturating_sub_for_slash`.
    pub fn checked_sub(self, other: Fixed) -> Result<Fixed, TransitionError> {
        let diff = self.0.checked_sub(other.0).ok_or(TransitionError::MathOverflow)?;
        Ok(Fixed(diff))
    }

    /// Saturating subtraction — ONLY for slashing operations.
    /// Clamps to zero rather than returning an error.
    /// Constitutionally restricted: must not be used for any balance math
    /// other than applying a slash penalty.
    pub fn saturating_sub_for_slash(self, slash_amount: Fixed) -> Fixed {
        Fixed(self.0.saturating_sub(slash_amount.0))
    }

    /// Returns true if this Fixed value is zero.
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Returns the zero Fixed value.
    pub fn zero() -> Fixed {
        Fixed(0)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests — use std only here
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_units_round_trips() {
        let f = Fixed::from_units(1).unwrap();
        assert_eq!(f.raw(), SCALE);
    }

    #[test]
    fn mul_scaled_basic() {
        // 2 * 3 = 6 units
        let a = Fixed::from_units(2).unwrap();
        let b = Fixed::from_units(3).unwrap();
        assert_eq!(a.mul_scaled(b).unwrap().raw(), 6 * SCALE);
    }

    #[test]
    fn div_by_zero_returns_error() {
        let a = Fixed::from_units(1).unwrap();
        let z = Fixed::zero();
        assert_eq!(a.div_scaled(z), Err(TransitionError::DivisionByZero));
    }

    #[test]
    fn saturating_slash_clamps_to_zero() {
        let balance = Fixed::from_units(5).unwrap();
        let huge_slash = Fixed::from_units(1000).unwrap();
        assert_eq!(balance.saturating_sub_for_slash(huge_slash), Fixed::zero());
    }

    #[test]
    fn from_canonical_str_valid() {
        assert!(Fixed::from_canonical_str("0").is_ok());
        assert!(Fixed::from_canonical_str("1000000000000").is_ok()); // 1 unit
    }

    #[test]
    fn from_canonical_str_rejects_float() {
        assert_eq!(
            Fixed::from_canonical_str("1.5"),
            Err(TransitionError::InvalidSerialization)
        );
    }

    #[test]
    fn from_canonical_str_rejects_leading_zero() {
        assert_eq!(
            Fixed::from_canonical_str("007"),
            Err(TransitionError::InvalidSerialization)
        );
    }
}
