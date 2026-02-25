# The Epoch State Structure (Step 3)

The `EpochState` struct is the ultimate expression of the domain's thermodynamics at a specific protocol timestamp. It is the object that the deterministic transition function `f(prev, payloads)` produces, and it is the object that 67% of the committee must identically hash and sign with BLS signatures.

## Architectural Decision: Root Commitments vs Full Materialized State
Given the requirement that browsers are full nodes, we must optimize for WASM memory constraints and replay speed. 

We choose **Option B: Merkle Root Commitments Only**. 
The `EpochState` struct does *not* contain the `HashMap<String, FixedPoint>` of all user balances. It only contains the `validator_set_root` and other Merkle commitments. 

### Why Commitments Only?
1. **WASM Memory Constraints:** If a domain grows to 1,000,000 identities, passing the entire materialized Map into the WASM sandboxed memory every transition will cause OOM (Out of Memory) crashes on mobile browsers.
2. **Stateless Replay:** By storing only roots, a node reconstructing state can fetch only the specific branches of the Merkle tree needed for the active transactions. This enables rapid, stateless verification.
3. **Fraud Proof Simplicity:** As defined in Phase 6, Fraud Proofs provide the Merkle inclusion proof for a *specific* invalid balance. If the state contained the entire map, verifying a Fraud Proof would require re-hashing the entire 1,000,000 item map. By using Merkle roots, the Fraud Proof only needs `log2(N)` hashes.

## The Rust Data Structure

```rust
use crate::fixed_point::FixedPoint;

/// Represents a 32-byte SHA-256 hash.
pub type Hash = [u8; 32];

/// The canonical immutable ledger state at the end of an Epoch.
/// This is the struct that is canonically serialized and BLS signed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochState {
    /// The canonical identifier for this exact state.
    /// Derived via SHA256(canonical_serialize(EpochState_without_state_root))
    pub state_root: Hash,

    /// The sequentially incrementing index of the epoch (0 is Genesis).
    pub epoch_number: u64,
    
    /// Binds this state to the thermodynamic arrow of time.
    /// MUST exactly match the `state_root` of the previous epoch.
    pub previous_root: Hash,
    
    /// The deterministic VDF output seed required for the NEXT sortition.
    /// This prevents look-ahead attacks.
    pub vdf_challenge_seed: Hash,

    /// Merkle Root of the Validator Registry (Pubkeys -> Balances & Roles).
    /// Represents the total active accountable citizenry.
    pub validator_set_root: Hash,
    
    /// Merkle Root of all historically valid ProofOfImpacts.
    /// Forms the immutable dataset of claims.
    pub impact_pool_root: Hash,
    
    /// Merkle Root of all active and historical VouchBonds.
    /// Tracks capital locked against reality.
    pub bond_pool_root: Hash,

    /// The cryptographically sealed Entropy Metric (Validator Churn %).
    /// Determines whether the domain is mathematically "alive" or mathematically "purged".
    pub entropy_metric_scaled: FixedPoint,
}
```

## The Transition Function & Chronology

The transition function cleanly takes the previous state and the *ordered* payload arrays. 

**Chronological Execution Invariant:**
To guarantee determinism, the WASM kernel must apply mathematical state transformations in this exact chronological sequence:
1. **Thermodynamic Decay:** Apply the `e^-0.0577` factor to all existing unlocked bounds and liquid balances FIRST. Decay computations use **Integer Division Truncation (toward zero)** to permanently burn fractional dust.
2. **Impact Processing:** Process and merge all authenticated `ProofOfImpact` objects into the Impact Merkle Tree.
3. **Bond Processing:** Process all `VouchBond` locks against the newly decayed balances. (You cannot bond decayed dust).
4. **Resolution processing & Yield:** (To be defined in the Economic Emission schema).
5. **State Commitment:** Calculate the new Merkle roots, construct the `EpochState` struct, and hash it to produce the final `state_root`.

### Struct Serialization Order (Frozen)
When canonicalizing the `EpochState` struct to compute the `state_root`, the fields MUST be ordered identically across all clients to prevent implicit omission forks. Serde auto-derivation without explicit test-vector checks is banned.
The frozen lexicographical order is:
1. `bond_pool_root`
2. `entropy_metric_scaled`
3. `epoch_number`
4. `impact_pool_root`
5. `previous_root`
6. `validator_set_root`
7. `vdf_challenge_seed`

```rust
pub fn apply_epoch(
    previous_state: &EpochState,
    lexicographically_sorted_impacts: Vec<ProofOfImpact>,
    lexicographically_sorted_bonds: Vec<VouchBond>,
    // The previous state does not contain the balances needed to 
    // validate the bonds. The caller (host) must provide the minimal 
    // Merkle branches (proofs) required to resolve these specific transactions.
    merkle_state_witnesses: StateWitnessBundle, 
) -> Result<EpochState, TransitionError>
```

This ensures the deterministic WASM kernel remains entirely memory-safe, computationally bounded, and purely functional.
