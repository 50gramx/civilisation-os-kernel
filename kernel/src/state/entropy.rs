//! Global entropy scalar computation.
//!
//! CONSTITUTIONAL FORMULA:
//! Global_Entropy = (Active_Bonded_Magnitude / Total_Supply)
//!               * (Unique_Active_Validators / Optimal_Validator_Count)
//!
//! Both ratios are computed as Fixed values (scaled to SCALE) before multiplication.
//! This prevents inflation when validators cartel or supply consolidates.

use crate::math::fixed::Fixed;
use crate::TransitionError;

/// Compute the Global_Entropy scalar given aggregated epoch statistics.
///
/// Arguments:
/// - `active_bonded_magnitude`: Sum of all VouchBond magnitudes in this epoch (Fixed).
/// - `total_supply`: Total circulating supply at epoch start (Fixed).
/// - `unique_active_validators`: Count of unique validators that participated.
/// - `optimal_validator_count`: The target validator set size from the Genesis Manifest.
///
/// Returns: a Fixed scalar ∈ [0, 1] (scaled to SCALE).
pub fn compute_entropy(
    active_bonded_magnitude: Fixed,
    total_supply: Fixed,
    unique_active_validators: u64,
    optimal_validator_count: u64,
) -> Result<Fixed, TransitionError> {
    if total_supply.is_zero() || optimal_validator_count == 0 {
        return Err(TransitionError::DivisionByZero);
    }
    // Ratio 1: bonded_ratio = Active_Bonded / Total_Supply
    let bonded_ratio = active_bonded_magnitude.div_scaled(total_supply)?;

    // Ratio 2: validator_ratio = Unique_Validators / Optimal_Count
    // Build both as Fixed from unit counts.
    let unique_val_fixed = Fixed::from_units(unique_active_validators as u128)?;
    let optimal_val_fixed = Fixed::from_units(optimal_validator_count as u128)?;
    let validator_ratio = unique_val_fixed.div_scaled(optimal_val_fixed)?;

    // Global_Entropy = bonded_ratio * validator_ratio
    bonded_ratio.mul_scaled(validator_ratio)
}
