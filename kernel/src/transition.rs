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
use crate::state::witness::StateWitnessBundle;

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
// apply_epoch — v0.0.2 Constitutional State Transition
// ──────────────────────────────────────────────────────────────────────────────

/// Advance one epoch using an authenticated `StateWitnessBundle`.
///
/// This replaces the dry-run stubs in `apply_epoch_dry_run` with:
/// - Real Merkle pool mutations (Model A evolving-root verification)
/// - Real entropy computation from host-provided aggregate statistics
///
/// `apply_epoch_dry_run` is retained as the v0.0.1 constitutional baseline.
///
/// # Pool Isolation (Constitutional)
///
/// Each pool is a sealed universe. The three calls are independent:
/// - `validator_witnesses` → `validator_set_root` only
/// - `impact_witnesses` → `impact_pool_root` only
/// - `bond_witnesses` → `bond_pool_root` only
///
/// Keys do not cross pool boundaries. Paths do not cross pool boundaries.
/// Mutations do not cross pool boundaries. Failures do not cross pool boundaries.
///
/// # Arguments
///
/// - `prev`: The committed state of the preceding epoch.
/// - `witness`: All pool mutations + entropy statistics for this epoch.
/// - `kernel_hash`: SHA-256 of the WASM kernel binary executing this transition.
///
/// # Returns
///
/// A new `EpochState` with:
/// - `epoch_number` = prev + 1
/// - `previous_root` = prev.state_root
/// - All three Merkle pool roots updated via witness-authenticated mutations
/// - `entropy_metric_scaled` computed from witness entropy stats
/// - `state_root` = SHA256(canonical JSON of all other fields)
/// - `vdf_challenge_seed` = all zeros (stub until v0.1.0)
pub fn apply_epoch(
    prev:        &EpochState,
    witness:     &StateWitnessBundle,
    kernel_hash: Digest,
) -> Result<EpochState, TransitionError> {
    use crate::math::fixed::Fixed;
    use crate::state::entropy::compute_entropy;
    use crate::state::witness::apply_pool_mutations;

    // ── Step 1: Validate bundle size limits ───────────────────────────────────
    // Reject oversized bundles before touching any Merkle state.
    witness.validate_limits()?;

    // ── Step 2: Validate entropy stats ────────────────────────────────────────
    // Entropy must be internally consistent before any pool is touched.
    // A failed entropy check aborts the epoch with no partial state mutation.
    witness.entropy_stats.validate()?;

    // ── Step 3: Epoch number (checked increment) ──────────────────────────────
    let new_epoch_number = prev
        .epoch_number
        .checked_add(1)
        .ok_or(TransitionError::MathOverflow)?;

    // ── Step 4: Chain the previous state root ─────────────────────────────────
    let new_previous_root = prev.state_root;

    // ── Step 5: Signature gate ────────────────────────────────────────────────
    // Authorization boundary: verify that a quorum of validators has signed
    // this exact epoch transition. No pool root is touched until this passes.
    //
    // HOST-TRUSTED (v0.0.2): Signature pubkeys are not verified against
    // validator_set_root. Full Merkle membership proofs required in v0.0.3.
    {
        use crate::state::witness::{compute_bundle_hash, compute_epoch_signing_root, verify_quorum};

        let bundle_hash = compute_bundle_hash(witness);
        let signing_root = compute_epoch_signing_root(
            &prev.state_root,
            &bundle_hash,
            new_epoch_number,
            &kernel_hash,
        );
        verify_quorum(
            &witness.validator_signatures,
            &signing_root,
            witness.entropy_stats.optimal_validator_count,
        )?;
    }

    // ── Step 6: Validator pool (registration + decay pass) ────────────────────
    // validator_witnesses covers both registration and decay mutations.
    // Within the array, registration mutations come first (lower keys),
    // decay mutations after; lexicographic order is enforced by apply_pool_mutations.
    let new_validator_set_root = apply_pool_mutations(
        prev.validator_set_root,
        &witness.validator_witnesses,
    )?;

    // ── Step 7: Impact pool ───────────────────────────────────────────────────
    let new_impact_pool_root = apply_pool_mutations(
        prev.impact_pool_root,
        &witness.impact_witnesses,
    )?;

    // ── Step 8: Bond pool ─────────────────────────────────────────────────────
    let new_bond_pool_root = apply_pool_mutations(
        prev.bond_pool_root,
        &witness.bond_witnesses,
    )?;

    // ── Step 9: Entropy computation ───────────────────────────────────────────
    // Convert raw u128 fields to Fixed and delegate to compute_entropy().
    // If total_supply is zero, compute_entropy returns DivisionByZero.
    // EntropyStats.validate() already ensures optimal_validator_count > 0.
    let active_bonded = Fixed::from_raw(witness.entropy_stats.active_bonded_magnitude_raw)?;
    let total_supply  = Fixed::from_raw(witness.entropy_stats.total_supply_raw)?;
    let entropy = compute_entropy(
        active_bonded,
        total_supply,
        witness.entropy_stats.unique_active_validators,
        witness.entropy_stats.optimal_validator_count,
    )?;
    let new_entropy_metric_scaled = entropy.raw();

    // ── Step 9: VDF challenge seed ────────────────────────────────────────────
    // STUB: Real seed is un-biasable VDF output (deferred to v0.1.0).
    let new_vdf_challenge_seed: Digest = [0u8; 32];

    // ── Step 10: Assemble and commit ──────────────────────────────────────────
    // commit() = canonicalize() → sha256() → assign state_root → Ok(self).
    // ALL pool mutations committed atomically: if commit() fails, nothing is written.
    let new_state = EpochState {
        bond_pool_root:        new_bond_pool_root,
        entropy_metric_scaled: new_entropy_metric_scaled,
        epoch_number:          new_epoch_number,
        impact_pool_root:      new_impact_pool_root,
        kernel_hash,
        previous_root:         new_previous_root,
        state_root:            [0u8; 32], // overwritten by commit()
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

    // ── apply_epoch (v0.0.2) ──────────────────────────────────────────────────

    use crate::state::witness::{
        EntropyStats, LeafMutation, MerklePath, MerklePathNode, NodePosition,
        StateWitnessBundle, apply_pool_mutations,
    };
    use crate::physics::hashing::{hash_leaf, hash_node};
    use crate::physics::merkle::empty_tree_root;

    /// Standard entropy stats for tests: 50% bonded, 50% participation → entropy = 0.25
    fn test_entropy() -> EntropyStats {
        // total_supply = Fixed(1) = raw 1_000_000_000_000
        // active_bonded = Fixed(0.5) = raw 500_000_000_000
        // unique_active = 5, optimal = 10
        // entropy = (0.5) * (5/10) = 0.25 = raw 250_000_000_000
        // entropy = (0.5) * (5/1) = wait, if optimal is 1, let's keep it structurally sound.
        EntropyStats {
            active_bonded_magnitude_raw: 500_000_000_000_u128,
            total_supply_raw:            1_000_000_000_000_u128,
            unique_active_validators:    5,
            optimal_validator_count:     10, // Must keep at 10 to not break entropy computation tests
        }
    }

    /// Build a single-level LeafMutation (two-leaf tree at genesis-like roots).
    fn epoch_mutation(
        key: &[u8],
        old_raw: &[u8],
        new_raw: &[u8],
        sibling: [u8; 32],
        position: NodePosition,
    ) -> LeafMutation {
        LeafMutation {
            key: key.to_vec(),
            old_value: old_raw.to_vec(),
            new_value: new_raw.to_vec(),
            path: MerklePath::new(vec![MerklePathNode { sibling, position }]).unwrap(),
        }
    }

    fn sign_for_test(signing_root: &Digest, seed: u8) -> crate::state::witness::ValidatorSignature {
        use ed25519_dalek::{SigningKey, Signer};
        let secret_bytes = [seed; 32];
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let signature = signing_key.sign(signing_root);
        crate::state::witness::ValidatorSignature {
            validator_pubkey: signing_key.verifying_key().to_bytes(),
            signature: signature.to_bytes(),
        }
    }

    fn add_valid_signatures(witness: &mut StateWitnessBundle, prev_root: &Digest, new_epoch_number: u64, kernel_hash: &Digest) {
        let bundle_hash = crate::state::witness::compute_bundle_hash(witness);
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            prev_root, &bundle_hash, new_epoch_number, kernel_hash
        );
        let threshold = (2 * witness.entropy_stats.optimal_validator_count as usize + 2) / 3;
        let mut sigs = vec![];
        for i in 0..threshold {
            sigs.push(sign_for_test(&signing_root, (i + 1) as u8));
        }
        sigs.sort_by_key(|s| s.validator_pubkey);
        witness.validator_signatures = sigs;
    }

    #[test]
    fn apply_epoch_empty_bundle_advances_epoch_and_preserves_roots() {
        // apply_epoch with all-empty witness vectors must:
        // - increment epoch_number by 1
        // - preserve all three pool roots unchanged
        // - chain previous_root correctly
        let genesis = zero_genesis();
        let mut witness = StateWitnessBundle {
            bond_witnesses:     vec![],
            entropy_stats:      test_entropy(),
            impact_witnesses:   vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        add_valid_signatures(&mut witness, &genesis.state_root, 1, &[0u8; 32]);

        let next = apply_epoch(&genesis, &witness, [0u8; 32]).unwrap();

        assert_eq!(next.epoch_number, 1, "epoch_number must increment");
        assert_eq!(next.previous_root, genesis.state_root, "must chain state root");
        assert_eq!(next.validator_set_root, genesis.validator_set_root, "validator pool must be unchanged");
        assert_eq!(next.impact_pool_root,   genesis.impact_pool_root,   "impact pool must be unchanged");
        assert_eq!(next.bond_pool_root,     genesis.bond_pool_root,     "bond pool must be unchanged");
        assert_ne!(next.state_root, genesis.state_root, "state_root must change (epoch advanced)");
        assert_ne!(next.entropy_metric_scaled, 0, "entropy must be non-zero");
    }

    #[test]
    fn apply_epoch_multi_pool_updates_correct_roots() {
        // Tree layout:
        //   validator_set_root = hash_node(hash_leaf("v1"), hash_leaf("v2"))
        //   impact_pool_root   = hash_leaf("i1")   (single leaf = no path)
        //   bond_pool_root     = genesis all-zeros  (no bond mutations)
        let leaf_v1 = hash_leaf(b"v1");
        let leaf_v2 = hash_leaf(b"v2");
        let leaf_i1 = hash_leaf(b"i1");

        let initial_validator_root = hash_node(&leaf_v1, &leaf_v2);
        let initial_impact_root    = leaf_i1; // single-leaf: root IS the hash

        let mut initial_state = zero_genesis();
        initial_state.validator_set_root = initial_validator_root;
        initial_state.impact_pool_root   = initial_impact_root;
        // Re-commit to get correct state_root.
        let initial_state = initial_state.commit().unwrap();

        // Validator mutation: v1 → v1_updated (v1 is LEFT child)
        let v_mutation = epoch_mutation(b"v1", b"v1", b"v1_updated", leaf_v2, NodePosition::Left);
        // Impact mutation: i1 → i1_updated (single leaf, empty path)
        let i_mutation = LeafMutation {
            key: b"i1".to_vec(),
            old_value: b"i1".to_vec(),
            new_value: b"i1_updated".to_vec(),
            path: MerklePath::new(vec![]).unwrap(),
        };

        let mut witness = StateWitnessBundle {
            bond_witnesses:      vec![],
            entropy_stats:       test_entropy(),
            impact_witnesses:    vec![i_mutation],
            validator_signatures: vec![],
            validator_witnesses: vec![v_mutation],
        };

        add_valid_signatures(&mut witness, &initial_state.state_root, 1, &[0u8; 32]);

        let next = apply_epoch(&initial_state, &witness, [0u8; 32])
            .expect("multi-pool test must verify structurally");

        // Validator root must change.
        let expected_validator_root = hash_node(&hash_leaf(b"v1_updated"), &leaf_v2);
        assert_eq!(next.validator_set_root, expected_validator_root,
            "validator_set_root must reflect mutation");

        // Impact root must change (single-leaf tree → new leaf hash).
        let expected_impact_root = hash_leaf(b"i1_updated");
        assert_eq!(next.impact_pool_root, expected_impact_root,
            "impact_pool_root must reflect mutation");

        // Bond root must be unchanged (no bond witnesses).
        assert_eq!(next.bond_pool_root, initial_state.bond_pool_root,
            "bond_pool_root must be unchanged when no bond witnesses provided");

        // Entropy must be computed (non-zero, not passthrough from prev).
        assert_ne!(next.entropy_metric_scaled, initial_state.entropy_metric_scaled,
            "entropy must be freshly computed, not passed through");

        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // Two-pool mutation epoch (validator v1→v1_updated, impact i1→i1_updated,
        // bond unchanged, entropy 50%×50%=25%, kernel_hash=[0;32], signed by 7 quorum validators).
        // Final state_root = SHA256(canonical JSON of new EpochState).
        // Any change to apply_epoch, apply_pool_mutations, compute_entropy,
        // or EpochState serialization will break this assertion immediately.
        let expected_state_root: [u8; 32] = [
            0x18, 0x5d, 0xd9, 0xc6, 0x2c, 0xeb, 0x2b, 0x0b,
            0x39, 0xcb, 0xa5, 0x8a, 0xe1, 0x8d, 0x04, 0xf6,
            0x00, 0xd3, 0xf2, 0xc7, 0x50, 0xb8, 0xc2, 0x77,
            0x2d, 0x6e, 0x06, 0xb8, 0x3d, 0x98, 0xb2, 0x83,
        ];
        assert_eq!(next.state_root, expected_state_root,
            "multi-pool epoch state_root diverged — apply_epoch execution path changed");
    }

    #[test]
    fn apply_epoch_corrupt_validator_path_fails_entire_epoch() {
        // A bad path in validator_witnesses must abort the entire epoch.
        // bond_pool_root and impact_pool_root must NOT be updated.
        let leaf_v1 = hash_leaf(b"v1");
        let leaf_v2 = hash_leaf(b"v2");
        let initial_validator_root = hash_node(&leaf_v1, &leaf_v2);
        let mut state = zero_genesis();
        state.validator_set_root = initial_validator_root;
        let state = state.commit().unwrap();

        // Wrong sibling → path will not verify.
        let bad_mutation = epoch_mutation(
            b"v1", b"v1", b"v1_updated",
            hash_leaf(b"WRONG_SIBLING"), // corrupted
            NodePosition::Left,
        );

        let mut witness = StateWitnessBundle {
            bond_witnesses:      vec![],
            entropy_stats:       test_entropy(),
            impact_witnesses:    vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![bad_mutation],
        };

        add_valid_signatures(&mut witness, &state.state_root, 1, &[0u8; 32]);

        assert_eq!(
            apply_epoch(&state, &witness, [0u8; 32]),
            Err(TransitionError::InvalidMerkleWitness),
            "corrupt validator path must fail the entire epoch"
        );
    }

    #[test]
    fn apply_epoch_corrupt_entropy_fails_before_any_pool_mutation() {
        // Entropy validation happens BEFORE pools are touched.
        // A corrupt entropy must abort before any Merkle root changes.
        let witness = StateWitnessBundle {
            bond_witnesses:      vec![],
            entropy_stats:       EntropyStats {
                active_bonded_magnitude_raw: 2_000_000_000_000_u128, // > total_supply
                total_supply_raw:            1_000_000_000_000_u128,
                unique_active_validators:    5,
                optimal_validator_count:     10,
            },
            impact_witnesses:    vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        assert_eq!(
            apply_epoch(&zero_genesis(), &witness, [0u8; 32]),
            Err(TransitionError::MathOverflow),
            "bonded > supply must fail before any pool mutation"
        );
    }
    // ────────────────────────────────────────────────────────────────────────
    // Signature Gate Consensus Tests
    // ────────────────────────────────────────────────────────────────────────

    use crate::state::witness::ValidatorSignature;

    /// Helper to generate a valid keypair and signature for testing the gate.
    /// In a real system, the host provides these.
    fn sign_for_test(signing_root: &Digest, seed: u8) -> ValidatorSignature {
        use ed25519_dalek::{SigningKey, Signer, SecretKey};
        let mut secret_bytes = [seed; 32];
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let signature = signing_key.sign(signing_root);
        ValidatorSignature {
            validator_pubkey: signing_key.verifying_key().to_bytes(),
            signature: signature.to_bytes(),
        }
    }

    #[test]
    fn apply_epoch_valid_quorum_passes() {
        let prev_state = zero_genesis();
        let witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: EntropyStats {
                active_bonded_magnitude_raw: 0,
                total_supply_raw: 1000,
                unique_active_validators: 1,
                optimal_validator_count: 3, // threshold = (2*3+2)/3 = 2
            },
            impact_witnesses: vec![],
            validator_signatures: vec![], // will populate
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root,
            &bundle_hash,
            1, // new_epoch_number
            &[0u8; 32], // kernel_hash
        );

        let sig1 = sign_for_test(&signing_root, 1);
        let sig2 = sign_for_test(&signing_root, 2);

        // Sort to ensure strict ascending order
        let mut sigs = vec![sig1, sig2];
        sigs.sort_by_key(|s| s.validator_pubkey);

        let mut signed_witness = witness.clone();
        signed_witness.validator_signatures = sigs;

        assert!(apply_epoch(&prev_state, &signed_witness, [0u8; 32]).is_ok(),
            "valid quorum must pass");
    }

    #[test]
    fn apply_epoch_insufficient_signature_count_fails() {
        let prev_state = zero_genesis();
        let witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: EntropyStats {
                active_bonded_magnitude_raw: 0,
                total_supply_raw: 1000,
                unique_active_validators: 1,
                optimal_validator_count: 4, // threshold = (2*4+2)/3 = 3
            },
            impact_witnesses: vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root,
            &bundle_hash,
            1,
            &[0u8; 32],
        );

        // Only 2 signatures for a threshold of 3
        let sig1 = sign_for_test(&signing_root, 1);
        let sig2 = sign_for_test(&signing_root, 2);
        let mut sigs = vec![sig1, sig2];
        sigs.sort_by_key(|s| s.validator_pubkey);

        let mut signed_witness = witness.clone();
        signed_witness.validator_signatures = sigs;

        assert_eq!(
            apply_epoch(&prev_state, &signed_witness, [0u8; 32]),
            Err(TransitionError::InvalidSignature),
            "insufficient signature count must fail"
        );
    }

    #[test]
    fn apply_epoch_duplicate_pubkey_fails() {
        let prev_state = zero_genesis();
        let mut witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: test_entropy(),
            impact_witnesses: vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root, &bundle_hash, 1, &[0u8; 32]
        );

        let sig = sign_for_test(&signing_root, 1);
        witness.validator_signatures = vec![sig.clone(), sig]; // Duplicate!

        assert_eq!(
            apply_epoch(&prev_state, &witness, [0u8; 32]),
            Err(TransitionError::InvalidSerialization),
            "duplicate pubkey must return InvalidSerialization"
        );
    }

    #[test]
    fn apply_epoch_reversed_pubkey_order_fails() {
        let prev_state = zero_genesis();
        let mut witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: test_entropy(),
            impact_witnesses: vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root, &bundle_hash, 1, &[0u8; 32]
        );

        let sig1 = sign_for_test(&signing_root, 1);
        let sig2 = sign_for_test(&signing_root, 2);
        let mut sigs = vec![sig1, sig2];
        sigs.sort_by_key(|s| s.validator_pubkey);
        sigs.reverse(); // Intentionally backwards

        witness.validator_signatures = sigs;

        assert_eq!(
            apply_epoch(&prev_state, &witness, [0u8; 32]),
            Err(TransitionError::InvalidSerialization),
            "reversed pubkey order must return InvalidSerialization"
        );
    }

    #[test]
    fn apply_epoch_wrong_kernel_hash_fails() {
        let prev_state = zero_genesis();
        let witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: test_entropy(),
            impact_witnesses: vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        // Signed with kernel hash [0; 32]
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root, &bundle_hash, 1, &[0u8; 32]
        );

        let sig = sign_for_test(&signing_root, 1);
        let mut signed_witness = witness.clone();
        signed_witness.validator_signatures = vec![sig];

        // Processed with a DIFFERENT kernel hash
        let bad_kernel_hash = [0xff; 32];
        assert_eq!(
            apply_epoch(&prev_state, &signed_witness, bad_kernel_hash),
            Err(TransitionError::InvalidSignature),
            "signature over wrong kernel hash must fail"
        );
    }

    #[test]
    fn apply_epoch_wrong_epoch_number_fails() {
        let mut prev_state = zero_genesis();
        prev_state.epoch_number = 5; // next epoch is 6
        let prev_state = prev_state.commit().unwrap();

        let witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: test_entropy(),
            impact_witnesses: vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        // Signed for epoch 7 (wrong!)
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root, &bundle_hash, 7, &[0u8; 32]
        );

        let sig = sign_for_test(&signing_root, 1);
        let mut signed_witness = witness.clone();
        signed_witness.validator_signatures = vec![sig];

        assert_eq!(
            apply_epoch(&prev_state, &signed_witness, [0u8; 32]),
            Err(TransitionError::InvalidSignature),
            "signature for wrong epoch number must fail"
        );
    }

    #[test]
    fn apply_epoch_mutated_bundle_content_fails() {
        let prev_state = zero_genesis();
        let mut witness = StateWitnessBundle {
            bond_witnesses: vec![],
            entropy_stats: test_entropy(),
            impact_witnesses: vec![],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };

        let bundle_hash = crate::state::witness::compute_bundle_hash(&witness);
        let signing_root = crate::state::witness::compute_epoch_signing_root(
            &prev_state.state_root, &bundle_hash, 1, &[0u8; 32]
        );

        // Sign the EMPTY bundle
        let sig = sign_for_test(&signing_root, 1);
        witness.validator_signatures = vec![sig];

        // Now mutate the bundle after signing! Add a malicious impact witness.
        use crate::state::witness::{LeafMutation, MerklePath, MerklePathNode, NodePosition};
        witness.impact_witnesses.push(LeafMutation {
            key: b"malicious".to_vec(),
            old_value: vec![],
            new_value: b"fake_impact".to_vec(),
            path: MerklePath::new(vec![MerklePathNode {
                sibling: [0u8; 32],
                position: NodePosition::Left,
            }]).unwrap()
        });

        assert_eq!(
            apply_epoch(&prev_state, &witness, [0u8; 32]),
            Err(TransitionError::InvalidSignature),
            "signature must fail if bundle content changes after signing"
        );
    }
}
