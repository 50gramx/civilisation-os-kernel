# StateWitnessBundle — Host ↔ Kernel Trust Boundary

**Version:** v0.0.2-draft  
**Status:** Design freeze required before implementation

---

## Purpose

The `StateWitnessBundle` is the single data structure that the host (node or browser) passes to the kernel when calling `apply_epoch`. It contains everything the kernel needs to independently verify a state transition without trusting any host-computed value.

If the kernel cannot derive a result from the witness alone, it must reject the transition.

---

## The Core Principle

> The kernel never trusts a root value provided by the host.  
> The kernel derives every root from leaves + Merkle paths.  
> A mismatch between derived root and host claim is immediately `Err(InvalidMerkleWitness)`.

This is what makes browser-based fraud proofs possible. A browser running the WASM kernel can receive the same `StateWitnessBundle` from a node, re-execute `apply_epoch`, and compare the resulting `state_root`. If they differ, the node is lying.

---

## Struct Layout

```rust
/// The complete set of data the host must provide for one epoch transition.
/// All fields are ordered alphabetically when canonically serialized (JCS).
pub struct StateWitnessBundle {
    /// Merkle witness data for the bond pool tree.
    pub bond_witnesses: Vec<LeafMutation>,

    /// Aggregate statistics required to compute the entropy metric.
    /// The kernel cannot derive these from Merkle proofs — they span the
    /// entire validator set. The host must provide them; the kernel verifies
    /// internal consistency (see EntropyStats constraints below).
    pub entropy_stats: EntropyStats,

    /// Merkle witness data for the impact pool tree.
    pub impact_witnesses: Vec<LeafMutation>,

    /// Merkle witness data for the validator set tree.
    /// Includes balance reads needed for bond validation.
    pub validator_witnesses: Vec<LeafMutation>,
}

/// A single leaf update in a Merkle tree.
/// The kernel uses this to:
/// 1. Verify old_leaf_hash + path == previous root (the leaf existed as claimed).
/// 2. Apply the mutation deterministically.
/// 3. Reconstruct the new root: new_leaf_hash + same_path.
pub struct LeafMutation {
    /// The canonical key identifying this leaf (JCS-canonicalized, ASCII key rules).
    /// For validator set: JCS-encoded Ed25519 public key (hex string).
    /// For impact pool: JCS-encoded impact ID (hex string of SHA-256 of ProofOfImpact).
    /// For bond pool: JCS-encoded bond ID (hex string of SHA-256 of VouchBond).
    pub key: Vec<u8>,

    /// Canonical bytes of the leaf value BEFORE this mutation.
    /// `[0u8; 0]` (empty) means the leaf did not exist (insertion case).
    pub old_value: Vec<u8>,

    /// Canonical bytes of the leaf value AFTER this mutation.
    /// `[0u8; 0]` (empty) means the leaf is being deleted (deletion case).
    pub new_value: Vec<u8>,

    /// The Merkle authentication path from this leaf to the tree root.
    pub path: MerklePath,
}

/// A Merkle authentication path: the sequence of sibling hashes from
/// leaf to root, with direction bits indicating which side each sibling is on.
///
/// Maximum length: MAX_MERKLE_DEPTH (40). Proofs longer than this are
/// immediately rejected with `TransitionError::InvalidMerkleWitness`.
pub struct MerklePath {
    /// Sibling nodes from leaf (index 0) up to root (index len-1).
    pub nodes: Vec<MerklePathNode>,
}

pub struct MerklePathNode {
    /// The sibling hash at this level of the tree.
    pub sibling: [u8; 32],
    /// Which side the CURRENT node is on at this level.
    /// Left means: current_node is the LEFT child, sibling is RIGHT.
    /// Right means: current_node is the RIGHT child, sibling is LEFT.
    pub position: NodePosition,
}

pub enum NodePosition {
    Left,   // node_hash(current, sibling)
    Right,  // node_hash(sibling, current)
}

/// Aggregate statistics used to compute the entropy metric.
/// The kernel verifies internal consistency but cannot independently
/// derive these values from Merkle proofs alone (they span all leaves).
///
/// TRUST BOUNDARY: These values are partially trusted from the host.
/// The kernel checks: active_bonded_magnitude <= total_supply.
/// It cannot verify the exact count of unique_active_validators
/// without iterating all validator leaves — which would require O(N) witnesses.
///
/// This is the one acknowledged host-trust surface in the kernel.
/// It is explicitly documented and will be tightened in v0.1.0 via
/// a validator set accumulator or VRF-based sampling proof.
pub struct EntropyStats {
    /// Sum of all VouchBond magnitudes active in this epoch (raw Fixed u128).
    pub active_bonded_magnitude_raw: u128,

    /// Total circulating supply at epoch start (raw Fixed u128).
    pub total_supply_raw: u128,

    /// Count of unique validators that submitted at least one payload.
    pub unique_active_validators: u64,

    /// The target validator set size from the Genesis Manifest.
    /// Must match the value committed in the genesis state.
    pub optimal_validator_count: u64,
}
```

---

## Empty Leaf Hashing (Frozen)

In this implementation, the following identity holds:

```
hash_leaf([]) = SHA256(0x00 || []) = SHA256([0x00]) = empty_tree_root()
```

This equality is **intentional** and **frozen**. Both constants share the leaf-domain prefix `0x00`. An insertion witness (`old_value = []`) produces `hash_leaf([])`, which equals `empty_tree_root()`. This means: an insert proof verifies that the path, given `empty_tree_root()` as the leaf hash, reproduces the prior pool root.

These two constants must be treated as **equal by definition**. Any future implementation that breaks this identity is a fork.

> **v0.0.2 restriction:** Non-membership proofs (proving a key does NOT exist) are not required in v0.0.2 because all pools are append-only in the alpha. A new validator, impact, or bond ID can only be inserted; it cannot replace a prior value at a different key. Deletion (`new_value = []`) is allowed for validator withdrawal only.

---

## Verification Algorithm (Kernel-Side)

### Key-to-Position Binding (Gap 1 — Closed)

The `LeafMutation.key` field is **committed inside `old_value`**. Every leaf value is the canonical JCS serialization of the entry, and the `key` field appears as the canonical identifier within that serialization. The kernel **must** extract and compare the key field from `old_value` against `LeafMutation.key`. A mismatch is `Err(InvalidSerialization)`.

This binding ensures: a valid Merkle path cannot be reused for a different key's leaf. The path proves position; the old_value proves key identity at that position.

### Multi-Mutation Path Model (Gap 2 — Model A Frozen)

**Model A is adopted. This is constitutional.**

When multiple mutations affect the same pool in one epoch:
- Each witness path is provided by the host **relative to the intermediate root** after all preceding mutations in execution order.
- The kernel applies mutations one at a time, in lexicographic key order.
- The root evolves with each mutation: `root_i+1 = reconstruct(new_leaf_hash_i, path_i)` where `path_i` was constructed against `root_i`.
- The host must pre-compute all intermediate paths accounting for prior mutations.

Model B (all paths relative to original root) is **rejected**. It requires non-overlap proofs that are more complex and less safe. Model A makes the host responsible for correct path construction, and the kernel verifiable at each step.

### Step-by-Step (Per LeafMutation)

```
1. Extract key_in_value from old_value (canonical key field).
   Assert key_in_value == LeafMutation.key.
   Err(InvalidSerialization) if mismatch.

2. Compute old_leaf_hash:
   if old_value is empty: old_leaf_hash = hash_leaf([]) = empty_tree_root()
   else: old_leaf_hash = hash_leaf(old_value)

3. Walk MerklePath from old_leaf_hash toward root:
   for each node in path.nodes (leaf → root direction):
     if position == Left:  current = hash_node(current, sibling)
     if position == Right: current = hash_node(sibling, current)

4. Assert current == current_pool_root  (evolving root, not prev_state root).
   Err(InvalidMerkleWitness) if mismatch.
   current_pool_root starts as prev_state.<pool>_root for the first mutation.

5. Compute new_leaf_hash:
   if new_value is empty: new_leaf_hash = hash_leaf([]) = empty_tree_root()
   else: new_leaf_hash = hash_leaf(new_value)

6. Walk same path with new_leaf_hash to reconstruct the new intermediate root.
   new_pool_root = reconstructed root.

7. Set current_pool_root = new_pool_root for the next mutation in this pool.

8. After all mutations: new EpochState.<pool>_root = final current_pool_root.
```

---

## Ordering Rules (Constitutional)

These are **frozen** — changing them is a fork.

### Leaf processing order

Within each pool (`validator_witnesses`, `impact_witnesses`, `bond_witnesses`), the kernel processes `LeafMutation` entries in **strictly ascending lexicographic order of `key`**.

This matches the sort order used by the Merkle tree's leaf insertion logic. Out-of-order witnesses are rejected with `Err(InvalidSerialization)`.

### Execution order across pools

The five-step execution sequence from `transition_executor_spec.md` is preserved:

1. Validator updates (`validator_witnesses`, registration pass)
2. Thermodynamic decay (applied via `validator_witnesses`, decay pass)
3. Impact processing (`impact_witnesses`)
4. Bond locking (`bond_witnesses`)
5. Entropy recomputation (`entropy_stats`)

A single `validator_witnesses` array covers both passes. The kernel processes all registration mutations first (new leaves or deletions), then all decay mutations. Within each pass, lexicographic order applies.

---

## Canonical Serialization of the Bundle

The `StateWitnessBundle` must itself be canonicalizable for fraud proof construction.

- All `Vec<u8>` fields that represent leaf values are serialized as their raw bytes (not hex strings — these are internal, not user-facing JSON).
- The bundle is serialized using the same `canonicalize()` function as `EpochState`.
- The canonical hash `SHA256(canonicalize(StateWitnessBundle))` is committed in the fraud proof schema.
- `NodePosition::Left` → JSON string `"left"`. `NodePosition::Right` → `"right"`.
- `MerklePath.nodes` is a JSON array — insertion order preserved (not sorted).

---

## Size Limits (Constitutional)

| Constraint | Value | Error on violation |
|---|---|---|
| `MerklePath.nodes.len()` | ≤ 40 (`MAX_MERKLE_DEPTH`) | `InvalidMerkleWitness` |
| `validator_witnesses.len()` | ≤ `MAX_PAYLOADS_PER_EPOCH` (10,000) | `PayloadLimitExceeded` |
| `impact_witnesses.len()` | ≤ `MAX_PAYLOADS_PER_EPOCH` | `PayloadLimitExceeded` |
| `bond_witnesses.len()` | ≤ `MAX_PAYLOADS_PER_EPOCH` | `PayloadLimitExceeded` |
| Combined witnesses total | ≤ `MAX_PAYLOADS_PER_EPOCH` | `PayloadLimitExceeded` |
| `key.len()` | ≤ 64 bytes | `InvalidSerialization` |
| `old_value.len()` / `new_value.len()` | ≤ 4096 bytes each | `InvalidSerialization` |

---

## The Acknowledged Trust Surface

The `EntropyStats` block is the **only** data in the entire bundle that the kernel cannot independently verify from Merkle proofs.

| Field | Kernel can verify? | How |
|---|---|---|
| `active_bonded_magnitude_raw` | Partially — must be ≤ `total_supply_raw` | Constraint check |
| `total_supply_raw` | No — requires summing all balance leaves | Host-trusted |
| `unique_active_validators` | No — requires counting participating validators | Host-trusted |
| `optimal_validator_count` | Yes — must match genesis manifest constant | Constant comparison |

**This is intentional for v0.0.2.** The kernel documents the limitation explicitly. It will be addressed in v0.1.0 via either:
- A **validator set accumulator** (running sum committed in `EpochState`) that allows the kernel to verify total supply without O(N) witnesses, or
- A **VRF-based sampling proof** that proves the validator participation rate without enumerating all validators.

Any system that pretends this trust surface doesn't exist is dangerous. This document names it.

---

## Witness Validity Invariants (Frozen)

These are the constitutional rules the kernel enforces before accepting any state transition. Violating any one is `Err(InvalidMerkleWitness)` or `Err(InvalidSerialization)` as noted.

1. **Key–value binding**: `LeafMutation.key` must equal the canonical key field extracted from `old_value`. (`Err(InvalidSerialization)`)
2. **No duplicate keys within a pool**: Within `validator_witnesses`, `impact_witnesses`, or `bond_witnesses`, no two `LeafMutation` entries may share the same `key`. (`Err(InvalidSerialization)`)
3. **No cross-pool key collisions**: The same key must not appear in more than one pool's witness array. Pool namespaces are disjoint. (`Err(InvalidSerialization)`)
4. **Lexicographic ordering enforced**: Within each pool, `LeafMutation` entries must be in strictly ascending lexicographic order of `key`. Out-of-order witnesses are rejected. (`Err(InvalidSerialization)`)
5. **Paths are relative to evolving root** (Model A): The first mutation's path must verify against `prev_state.<pool>_root`. Each subsequent mutation's path must verify against the root reconstructed by the prior mutation.
6. **`hash_leaf([]) == empty_tree_root()`**: Insert witnesses (`old_value = []`) must produce a path that verifies against the current root using this constant. No alternative encoding is accepted.
7. **Final reconstructed root is committed**: After all mutations in a pool, the final `current_pool_root` is the value written into the new `EpochState`. The host may not provide a different value.
8. **Path length ≤ MAX_MERKLE_DEPTH (40)**: Any path longer than 40 nodes is rejected. (`Err(InvalidMerkleWitness)`)
9. **Totals check on EntropyStats**: `active_bonded_magnitude_raw ≤ total_supply_raw`. (`Err(MathOverflow)` if violated)
10. **`optimal_validator_count` matches genesis constant**: Must equal the value in the genesis manifest. (`Err(InvalidSerialization)`)

---

## What Does NOT Belong in the Bundle

- Raw signatures (these belong on `ProofOfImpact` and `VouchBond` structs, not the witness)
- The VDF proof (passed as a separate `vdf_proof: &[u8]` parameter)
- Any host-computed new root values (the kernel derives these itself)
- Any network metadata (peer IDs, timestamps, IP addresses)
- Any data not required to derive the new `EpochState`

---

## v0.0.2 Implementation Checklist

Before any crypto is added, the following Rust types must be defined in `kernel/src/state/witness.rs`:

- [ ] `MerklePathNode` struct
- [ ] `MerklePath` struct with `MAX_MERKLE_DEPTH` enforcement
- [ ] `LeafMutation` struct with size limit enforcement
- [ ] `EntropyStats` struct with constraint checks (`active_bonded ≤ total_supply`)
- [ ] `StateWitnessBundle` struct
- [ ] `MerklePath::verify(leaf_hash, expected_root) -> Result<(), TransitionError>`
- [ ] `MerklePath::reconstruct_root(new_leaf_hash) -> Digest`
- [ ] A pinned constitutional test vector for `MerklePath::verify`

The `apply_epoch` signature (replacing `apply_epoch_dry_run`) only expands after all of the above types have passing test vectors.
