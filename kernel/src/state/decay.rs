//! Thermodynamic decay: apply DECAY_FACTOR_PER_EPOCH to a balance.
//!
//! CONSTITUTIONAL:
//! - Decay applies to liquid unlocked balances only.
//! - Identity iteration MUST be in strictly ascending lexicographical order of public keys.
//! - Rounding: integer division truncation (floor for unsigned). Dust is burned.
//! - Decay uses mul_scaled, not raw multiplication.

use crate::math::fixed::Fixed;
use crate::TransitionError;

/// Decay factor per epoch scaled to SCALE (10^12).
/// Represents 0.943932824245 → ~5.6% monthly decay at 30-day epochs.
/// Precomputed offline, truncated at the 12th decimal, then multiplied by SCALE.
pub const DECAY_FACTOR_SCALED: u128 = 943_932_824_245;

/// Returns the decay factor as a Fixed value.
pub fn decay_factor() -> Result<Fixed, TransitionError> {
    Fixed::from_raw(DECAY_FACTOR_SCALED)
}

/// Apply one epoch of thermodynamic decay to a balance.
/// Returns the decayed balance (dust remainder is burned).
pub fn apply_decay(balance: Fixed) -> Result<Fixed, TransitionError> {
    let factor = decay_factor()?;
    balance.mul_scaled(factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::fixed::SCALE;

    #[test]
    fn decay_reduces_balance() {
        let balance = Fixed::from_units(1000).unwrap();
        let decayed = apply_decay(balance).unwrap();
        // 1000 units * DECAY_FACTOR_SCALED / SCALE
        // = (1000 * SCALE) * DECAY_FACTOR_SCALED / SCALE
        // = 1000 * DECAY_FACTOR_SCALED
        let expected_raw = 1000u128 * DECAY_FACTOR_SCALED;
        assert!(decayed.raw() < balance.raw(), "decay must reduce balance");
        // Allow ±1 unit of truncation difference from integer division.
        assert!(
            (decayed.raw() as i128 - expected_raw as i128).abs() <= 1,
            "decay amount off: got {}, expected {}",
            decayed.raw(),
            expected_raw
        );
    }
}
