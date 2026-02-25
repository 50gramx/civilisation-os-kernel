//! `apply_epoch_dry_run` — Deterministic state transition, v0.0.1-alpha.
//!
//! # What This Is
//!
//! The minimal, deterministic epoch transition function.
//! It advances the epoch counter, chains the state hash, and recomputes the
//! state root — without requiring signatures, VDF proofs, or Merkle witnesses.
//!
//! # What Is Real (v0.0.1-alpha)
//!
//! - Epoch counter increment (checked, never wrapping)
//! - Previous root chaining (thermodynamic arrow of time)
//! - Payload count bounding (MAX_PAYLOADS_PER_EPOCH enforced)
//! - Kernel hash commitment (caller provides the binary hash)
//! - Empty-epoch Merkle root passthrough (no payload = no root change)
//! - Canonical serialization → SHA-256 → new state_root
//!
//! # What Is Stubbed (returns TransitionError::NotYetImplemented)
//!
//! - VDF SNARK proof verification (requires arkworks, deferred to v0.0.2)
//! - Ed25519 signature verification on impacts and bonds
//! - Per-identity thermodynamic decay (requires Merkle witnesses from host)
//! - Entropy metric recalculation (requires aggregate supply stats from host)
//! - Bond and impact Merkle tree updates
//! - Yield processing
//!
//! # Execution Sequence (Frozen for v0.0.1-alpha)
//!
//! ```text
//! 1. Validate inputs (payload count ≤ MAX_PAYLOADS_PER_EPOCH)
//! 2. VDF verification          → STUB (always Ok in dry_run)
//! 3. Decompose previous state
//! 4. epoch_number              → prev.epoch_number + 1  (checked)
//! 5. previous_root             → prev.state_root
//! 6. Decay (empty round)       → STUB (Merkle roots pass through)
//! 7. Impacts                   → STUB (Merkle root passes through)
//! 8. Bonds                     → STUB (Merkle root passes through)
//! 9. Yield                     → STUB
//! 10. Entropy                  → STUB (passes through)
//! 11. kernel_hash              → caller-provided
//! 12. vdf_challenge_seed       → STUB [0u8; 32]
//! 13. Canonical serialization → SHA-256 → state_root
//! 14. Return Ok(new_state)
//! ```
//!
//! The stub steps will be replaced one by one as v0.0.1 → v0.0.2 → v0.1 progress.
//! Each replacement requires a new pinned test vector.

use crate::TransitionError;
use crate::physics::hashing::Digest;
use crate::state::epoch::{EpochState, MAX_PAYLOADS_PER_EPOCH};

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Advance one epoch deterministically without payload processing.
///
/// # Arguments
///
/// - `prev`: The previous epoch's committed state (state_root must be valid).
/// - `payload_count`: The number of payloads proposed for this epoch.
///   Rejected immediately if it exceeds `MAX_PAYLOADS_PER_EPOCH`.
/// - `kernel_hash`: SHA-256 of the WASM kernel binary executing this transition.
///   Callers must compute this at startup and pass it in.
///   Using `[0u8; 32]` is valid in tests (no binary present).
///
/// # Returns
///
/// A new `EpochState` with:
/// - `epoch_number` = prev + 1
/// - `previous_root` = prev.state_root
/// - `state_root` = SHA256(canonical JSON of all other fields)
/// - All Merkle roots unchanged from previous epoch (no payloads processed)
/// - `entropy_metric_scaled` unchanged (stub; no supply stats)
/// - `vdf_challenge_seed` = all zeros (stub; no VDF proof available)
pub fn apply_epoch_dry_run(
    prev: &EpochState,
    payload_count: usize,
    kernel_hash: Digest,
) -> Result<EpochState, TransitionError> {
    // ── Step 1: Validate payload ceiling ──────────────────────────────────────
    if payload_count > MAX_PAYLOADS_PER_EPOCH {
        return Err(TransitionError::PayloadLimitExceeded);
    }

    // ── Step 2: VDF verification ──────────────────────────────────────────────
    // STUB: VDF SNARK verification requires arkworks (deferred to v0.0.2).
    // In dry_run, we accept any epoch transition without time-lock proof.

    // ── Step 3: Epoch number increment ────────────────────────────────────────
    // Must not overflow. A u64 epoch counter would exhaust after 584 billion years
    // at 30-day epochs, but we still enforce the checked contract.
    let new_epoch_number = prev
        .epoch_number
        .checked_add(1)
        .ok_or(TransitionError::MathOverflow)?;

    // ── Step 4: Chain the previous state root ─────────────────────────────────
    // This is the thermodynamic arrow of time. Skipping or swapping previous_root
    // breaks chain continuity and makes fraud proofs impossible.
    let new_previous_root = prev.state_root;

    // ── Step 5–9: Payload processing ──────────────────────────────────────────
    // STUB: All Merkle roots pass through unchanged.
    // When payload processing is implemented, these will be replaced with
    // actual Merkle tree updates driven by Merkle witnesses and signature verification.
    let new_validator_set_root = prev.validator_set_root;
    let new_impact_pool_root   = prev.impact_pool_root;
    let new_bond_pool_root     = prev.bond_pool_root;

    // ── Step 10: Entropy metric ───────────────────────────────────────────────
    // STUB: Entropy requires aggregate active-bonded and total-supply values
    // from the host. In dry_run we preserve the previous epoch's entropy.
    let new_entropy_metric_scaled = prev.entropy_metric_scaled;

    // ── Step 11: VDF challenge seed ───────────────────────────────────────────
    // STUB: The real seed is derived from the VDF SNARK output.
    // All-zero in dry_run. A real implementation replaces this with the
    // un-biasable VDF output extracted from the verified SNARK proof.
    let new_vdf_challenge_seed: Digest = [0u8; 32];

    // ── Step 12: Assemble and commit ──────────────────────────────────────────
    // Build the new state and compute state_root via canonical serialization.
    // `commit()` = canonical_bytes() → sha256() → assign state_root → Ok(self).
    let new_state = EpochState {
        bond_pool_root:        new_bond_pool_root,
        entropy_metric_scaled: new_entropy_metric_scaled,
        epoch_number:          new_epoch_number,
        impact_pool_root:      new_impact_pool_root,
        kernel_hash,
        previous_root:         new_previous_root,
        state_root:            [0u8; 32], // will be overwritten by commit()
        validator_set_root:    new_validator_set_root,
        vdf_challenge_seed:    new_vdf_challenge_seed,
    };

    new_state.commit()
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::epoch::EpochState;

    fn zero_genesis() -> EpochState {
        EpochState::genesis()
    }

    // ── Basic correctness ─────────────────────────────────────────────────────

    #[test]
    fn epoch_number_increments_by_one() {
        let g = zero_genesis();
        let next = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        assert_eq!(next.epoch_number, 1);
    }

    #[test]
    fn previous_root_chains_to_genesis_state_root() {
        let g = zero_genesis();
        let genesis_root = g.state_root;
        let next = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        assert_eq!(next.previous_root, genesis_root,
            "previous_root must equal the prior epoch's state_root");
    }

    #[test]
    fn state_root_is_not_zero_after_commit() {
        let g = zero_genesis();
        let next = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        assert_ne!(next.state_root, [0u8; 32],
            "commit() must overwrite the placeholder state_root");
    }

    #[test]
    fn state_root_changes_from_genesis() {
        // Epoch 1 state root must differ from genesis state root.
        let g = zero_genesis();
        let next = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        assert_ne!(next.state_root, g.state_root,
            "advancing an epoch must produce a new state_root");
    }

    // ── Determinism ───────────────────────────────────────────────────────────

    #[test]
    fn same_inputs_produce_identical_outputs() {
        let g = zero_genesis();
        let a = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        let b = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        assert_eq!(a, b, "apply_epoch_dry_run must be deterministic");
    }

    #[test]
    fn chained_transitions_are_deterministic() {
        // Apply 5 epochs in sequence. Re-applying from genesis must give the same chain.
        let g = zero_genesis();
        let mut state = g.clone();
        for _ in 0..5 {
            state = apply_epoch_dry_run(&state, 0, [0u8; 32]).unwrap();
        }
        let final_root_a = state.state_root;

        // Re-run from scratch.
        let mut state = g;
        for _ in 0..5 {
            state = apply_epoch_dry_run(&state, 0, [0u8; 32]).unwrap();
        }
        assert_eq!(state.state_root, final_root_a,
            "chained transitions must be deterministic");
        assert_eq!(state.epoch_number, 5);
    }

    // ── Payload ceiling enforcement ───────────────────────────────────────────

    #[test]
    fn payload_count_at_limit_is_accepted() {
        let g = zero_genesis();
        assert!(apply_epoch_dry_run(&g, MAX_PAYLOADS_PER_EPOCH, [0u8; 32]).is_ok());
    }

    #[test]
    fn payload_count_over_limit_is_rejected() {
        let g = zero_genesis();
        assert_eq!(
            apply_epoch_dry_run(&g, MAX_PAYLOADS_PER_EPOCH + 1, [0u8; 32]),
            Err(TransitionError::PayloadLimitExceeded),
            "payload count exceeding MAX_PAYLOADS_PER_EPOCH must be rejected"
        );
    }

    // ── kernel_hash binding ───────────────────────────────────────────────────

    #[test]
    fn different_kernel_hash_produces_different_state_root() {
        let g = zero_genesis();
        let state_a = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        let state_b = apply_epoch_dry_run(&g, 0, [1u8; 32]).unwrap();
        assert_ne!(state_a.state_root, state_b.state_root,
            "kernel_hash must influence state_root — different binary = different chain");
    }

    // ── Chain continuity ──────────────────────────────────────────────────────

    #[test]
    fn chain_epoch_number_is_monotonically_increasing() {
        let g = zero_genesis();
        let e1 = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        let e2 = apply_epoch_dry_run(&e1, 0, [0u8; 32]).unwrap();
        let e3 = apply_epoch_dry_run(&e2, 0, [0u8; 32]).unwrap();
        assert!(g.epoch_number < e1.epoch_number);
        assert!(e1.epoch_number < e2.epoch_number);
        assert!(e2.epoch_number < e3.epoch_number);
        assert_eq!(e3.epoch_number, 3);
    }

    #[test]
    fn chain_links_are_intact() {
        // Each epoch's previous_root must equal the prior epoch's state_root.
        let g = zero_genesis();
        let e1 = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();
        let e2 = apply_epoch_dry_run(&e1, 0, [0u8; 32]).unwrap();
        assert_eq!(e1.previous_root, g.state_root);
        assert_eq!(e2.previous_root, e1.state_root);
    }

    // ── Pinned constitutional vector ──────────────────────────────────────────

    #[test]
    fn epoch_1_state_root_is_pinned() {
        // CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // SHA-256 of the canonical JSON of epoch 1 state, given:
        //   - genesis as prev_state (all-zero Merkle roots, epoch_number=0)
        //   - payload_count = 0
        //   - kernel_hash = [0u8; 32]
        //
        // Any change to apply_epoch_dry_run, EpochState serialization, or
        // the SHA-256 implementation will break this assertion immediately.
        let g = zero_genesis();
        let e1 = apply_epoch_dry_run(&g, 0, [0u8; 32]).unwrap();

        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // SHA-256 of the canonical JSON of epoch 1 state, given:
        //   prev = genesis (all-zero Merkle roots, epoch_number=0)
        //   payload_count = 0
        //   kernel_hash = [0u8; 32]
        //
        // Any change to apply_epoch_dry_run, EpochState serialization, sha256,
        // or canonical_json will break this assertion and signal a chain fork.
        let expected: [u8; 32] = [
            0x10, 0xdc, 0x6e, 0x69, 0x48, 0x43, 0xa9, 0xa3,
            0x81, 0x3f, 0xec, 0xb4, 0x91, 0x99, 0xf5, 0xf8,
            0x1a, 0xb6, 0x1d, 0xa2, 0x0f, 0xe5, 0x36, 0xa0,
            0x9d, 0xb3, 0xb1, 0xfb, 0xf1, 0x90, 0x8e, 0xa1,
        ];
        assert_eq!(e1.state_root, expected,
            "epoch 1 state_root diverged — execution path changed");
        assert_eq!(e1.epoch_number, 1);
        assert_eq!(e1.previous_root, g.state_root);
    }

    // ── Time-amplified chain replay ────────────────────────────────────────────

    #[test]
    fn hundred_epoch_chain_is_deterministic_across_two_runs() {
        // Run the same 100-epoch chain twice independently from genesis.
        // Both runs must produce identical state_roots at every epoch.
        // This is the primary time-amplified determinism proof.
        let mut a = zero_genesis();
        for _ in 0..100 {
            a = apply_epoch_dry_run(&a, 0, [0u8; 32]).unwrap();
        }
        let mut b = zero_genesis();
        for _ in 0..100 {
            b = apply_epoch_dry_run(&b, 0, [0u8; 32]).unwrap();
        }
        assert_eq!(a.state_root, b.state_root,
            "100-epoch chain must produce identical roots across independent runs");
        assert_eq!(a.epoch_number, 100);
    }

    #[test]
    fn hundred_epoch_chain_all_links_are_intact() {
        // Verify the chain linkage invariant across 100 sequential epochs:
        // each epoch's previous_root must equal the prior epoch's state_root.
        // A single broken link would be undetectable without this exhaustive check.
        let mut prev = zero_genesis();
        for i in 1..=100 {
            let next = apply_epoch_dry_run(&prev, 0, [0u8; 32]).unwrap();
            assert_eq!(
                next.previous_root, prev.state_root,
                "chain link broken at epoch {i}: previous_root != prior state_root"
            );
            assert_eq!(next.epoch_number, i as u64);
            prev = next;
        }
    }

    #[test]
    fn epoch_100_state_root_is_pinned() {
        // CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // SHA-256 of the canonical JSON of epoch 100 state, given:
        //   100 sequential apply_epoch_dry_run from genesis
        //   payload_count = 0 at every epoch
        //   kernel_hash = [0u8; 32] at every epoch
        //
        // Proves time-amplified determinism: any drift in serialization,
        // arithmetic, or execution logic surfaces after at most 100 epochs.
        let mut state = zero_genesis();
        for _ in 0..100 {
            state = apply_epoch_dry_run(&state, 0, [0u8; 32]).unwrap();
        }
        assert_eq!(state.epoch_number, 100);
        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // SHA-256 of the canonical JSON of epoch 100, from genesis with:
        //   payload_count = 0, kernel_hash = [0u8; 32] at every epoch.
        // Any execution drift surfaces within 100 epochs.
        let expected: [u8; 32] = [
            0x23, 0x86, 0x15, 0xdb, 0x67, 0x8a, 0xcd, 0x7b,
            0xe8, 0x46, 0x0b, 0x8d, 0xd2, 0x50, 0x15, 0xf9,
            0x56, 0x06, 0x70, 0xa1, 0xac, 0x17, 0xd0, 0x83,
            0x6f, 0xae, 0x6a, 0x42, 0x72, 0xb3, 0x57, 0x99,
        ];
        assert_eq!(state.state_root, expected, "epoch 100 chain diverged — execution drift detected");
    }
}
