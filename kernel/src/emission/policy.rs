//! EmissionPolicy trait: the interface between the physics kernel and economic policy.
//!
//! The kernel compiles and operates correctly with ZeroEmission plugged in.
//! SublinearBondEmission is injected only after adversarial simulation proves stability.
//!
//! CONSTITUTIONAL FORMULA (for implementors):
//!   minted = isqrt[(Bond_Magnitude * Lock_Duration) / SCALE] * Global_Entropy
//!
//! Implementation order:
//!   1. checked_mul: Bond_Magnitude.raw() * Lock_Duration (u128)
//!   2. checked_div by SCALE
//!   3. isqrt of result
//!   4. Fixed::from_raw(isqrt_result)
//!   5. mul_scaled with Global_Entropy

use crate::math::fixed::Fixed;
use crate::TransitionError;

/// The emission policy interface.
/// The kernel never calls any method here during Physics Layer validation.
pub trait EmissionPolicy {
    /// Calculate tokens minted for a single VouchBond.
    ///
    /// Arguments:
    /// - `bond_magnitude`: The locked accountability magnitude (Fixed, ≥ MIN_BOND_MAGNITUDE).
    /// - `lock_duration_epochs`: How many epochs the bond is locked.
    /// - `global_entropy`: The computed entropy scalar for this epoch (Fixed ∈ [0, 1]).
    fn calculate_bond_mint(
        &self,
        bond_magnitude: Fixed,
        lock_duration_epochs: u64,
        global_entropy: Fixed,
    ) -> Result<Fixed, TransitionError>;

    /// Calculate the validator fee from a completed epoch's total minted amount.
    /// Nominally a fraction (e.g. 10%) of total minted, redirected to the active committee.
    fn calculate_validator_fee(&self, total_epoch_minted: Fixed) -> Result<Fixed, TransitionError>;
}
