# The Transition Executor (Step 4 & VDF Verification)

The execution engine is the beating heart of the deterministic physics model. It is a strictly pure function compiled inside the WASM sandbox.

This document outlines the pipeline for the `apply_epoch` function and addresses the vital architectural boundary regarding Verifiable Delay Function (VDF) SNARK proofs.

## 1. The VDF Validation Boundary

The VDF guarantees time has passed (creating the 30-day epoch interval) and provides an un-biasable random seed for the next sortition. The VDF outputs a SNARK proof verifying that the squarings were performed.

**Where does the SNARK verification happen?**
We have two options:
*   A) Trust the Flutter Host Shell: Flutter runs a native platform SNARK verifier and passes a boolean `true` into the WASM kernel.
*   B) WASM Autonomous Verification: The WASM kernel is passed the raw SNARK bytes and performs the elliptic curve pairings itself.

**Decision: Option B (WASM Autonomous Verification).**
If the WASM kernel blindly trusts a boolean from the Host, a corrupted Host binary could feed the kernel a fake `true` boolean, causing the WASM sandbox to falsely transition the state based on a fraudulent VDF. The deterministic kernel must be cryptographically autonomous. It must contain its own SNARK verifier (e.g., using `arkworks` Rust libraries compiled to WASM) and independently reject invalid epoch transitions.

## 2. The `apply_epoch` Pipeline

The signature is functionally pure, relying entirely on the provided inputs (the previous state, the ordered payloads, and the Merkle witnesses provided by the Host storage).

### 2a. Computational Ceilings (DOS Protection)
To prevent a malicious host or P2P layer from feeding an infinitely large block to crash the WASM sandbox (OOM) or exhaust execution time, the kernel enforces strict, hardcoded limits before processing begins:
*   **MAX_PAYLOADS_PER_EPOCH:** `10,000` (Combined `ProofOfImpact` and `VouchBond` items). Any proposed array exceeding this size is instantly rejected.
*   **MAX_MERKLE_DEPTH:** `40` (Allows for $2^{40}$ active identities, far exceeding global population, while bounding hash iteration). Witness paths exceeding this depth throw a `TransitionError`. 

```rust
pub fn apply_epoch(
    previous_state: &EpochState,
    lexicographically_sorted_impacts: Vec<ProofOfImpact>,
    lexicographically_sorted_bonds: Vec<VouchBond>,
    merkle_state_witnesses: StateWitnessBundle, 
    vdf_snark_proof: &[u8], // The proof proving the passage of time
) -> Result<EpochState, TransitionError>
```

### The Chronological Execution Sequence:

The function executes sequentially. Any failure (e.g., invalid signature, arithmetic overflow, invalid Merkle witness path, invalid SNARK) triggers an immediate `TransitionError()`. 

1.  **VDF Verification & Sortition Preparation:**
    *   Verify `vdf_snark_proof` against the `previous_state.vdf_challenge_seed`.
    *   Extract the new un-biasable `new_vdf_challenge_seed`.
2.  **Validator Registration:**
    *   Process any new validator key registrations or withdrawals.
3.  **Thermodynamic Decay:**
    *   Iterate through the `merkle_state_witnesses` for every active identity.
    *   *Ordering Constraint:* Identities MUST be processed in strictly ascending lexicographical order of their JCS-canonical public keys. Even though decay multiplication is theoretically commutative and parallelizable, sequential deterministic ordering guarantees that any intermediate accumulation or dust-burning calculations execute identically across all nodes.
    *   Apply `DECAY_FACTOR_PER_EPOCH` to all unlocked liquid balances using Integer Division Truncation. *(Crucial: Decay applies to the balances as they existed at the exact end of the previous epoch, before any new impacts or bonds are considered).*
4.  **Impact Processing:**
    *   Verify Ed25519 signatures on all `ProofOfImpact` objects.
    *   Deduplicate identical impacts.
    *   Merge valid impacts into the new `impact_pool_root`.
5.  **Bond Locking:**
    *   Verify Ed25519 signatures on all `VouchBonds`.
    *   Deduct the `staked_weight` from the identity's newly-decayed liquid balance. (If the user tries to bond more than their decayed balance, the specific transaction is dropped).
    *   Merge valid bonds into the new `bond_pool_root`.
6.  **Yield Processing (Stubbed):**
    *   Calculate resolution outcomes based on the Emission Economic Model (to be defined in Step 7).
7.  **Calculate Entropy Constraint:**
    *   Compare the active validator churn against the established domains parameters to calculate the new `entropy_metric_scaled`.
8.  **Compute Self-Committing State Root:**
    *   Construct the proposed `EpochState` struct.
    *   Perform a canonical JCS Serialization of the struct (excluding the `state_root` field).
    *   Calculate SHA-256 hash.
    *   Assign the hash to `state_root`.
9.  **Return New Epoch:**
    *   `Ok(new_epoch_state)`
