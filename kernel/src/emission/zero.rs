//! ZeroEmission: the default emission policy for the Physics Layer validation phase.
//! All mint calculations return Fixed::zero(). No tokens are created.
//! This allows the kernel to prove deterministic replay without economic contamination.

use crate::emission::policy::EmissionPolicy;
use crate::math::fixed::Fixed;
use crate::TransitionError;

pub struct ZeroEmission;

impl EmissionPolicy for ZeroEmission {
    fn calculate_bond_mint(
        &self,
        _bond_magnitude: Fixed,
        _lock_duration_epochs: u64,
        _global_entropy: Fixed,
    ) -> Result<Fixed, TransitionError> {
        Ok(Fixed::zero())
    }

    fn calculate_validator_fee(
        &self,
        _total_epoch_minted: Fixed,
    ) -> Result<Fixed, TransitionError> {
        Ok(Fixed::zero())
    }
}
