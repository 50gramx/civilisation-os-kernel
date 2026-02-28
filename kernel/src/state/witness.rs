//! `StateWitnessBundle` — Host ↔ Kernel trust boundary types.
//!
//! Implements the data structures defined in `docs/specs/witness_schema.md`.
//! All field names, ordering rules, and size constraints are cross-verified
//! against that document. Any divergence from the spec is a protocol bug.
//!
//! # Position Semantics (read carefully)
//!
//! `NodePosition::Left` means the CURRENT node is the LEFT child.
//! Therefore: `parent = hash_node(current, sibling)`.
//!
//! `NodePosition::Right` means the CURRENT node is the RIGHT child.
//! Therefore: `parent = hash_node(sibling, current)`.
//!
//! This matches `witness_schema.md §Verification Algorithm` exactly.
//! The mnemonic: the position names WHERE the current node sits, not where
//! the sibling sits.
//!
//! # Evolving Root Model (Model A — Constitutional)
//!
//! When multiple `LeafMutation` entries modify the same pool:
//! - The first mutation's path verifies against `prev_state.<pool>_root`.
//! - Each subsequent mutation's path verifies against the root produced
//!   by the preceding mutation's `reconstruct_root()`.
//! - The host is responsible for constructing paths relative to intermediate
//!   roots. Model B (paths relative to original root) is rejected.

use crate::TransitionError;
use crate::physics::hashing::{Digest, sha256, hash_leaf, hash_node};
use crate::physics::merkle::MAX_MERKLE_DEPTH;
use crate::state::epoch::MAX_PAYLOADS_PER_EPOCH;

// ──────────────────────────────────────────────────────────────────────────────
// Constitutional constants
// ──────────────────────────────────────────────────────────────────────────────

/// Maximum byte length of a leaf mutation key.
/// From `witness_schema.md §Size Limits`.
pub const MAX_KEY_BYTES: usize = 64;

/// Maximum byte length of a leaf mutation value (old or new).
/// From `witness_schema.md §Size Limits`.
pub const MAX_VALUE_BYTES: usize = 4096;

// ──────────────────────────────────────────────────────────────────────────────
// NodePosition
// ──────────────────────────────────────────────────────────────────────────────

/// Which side of its parent the CURRENT node occupies.
///
/// `Left`  → current is left child  → `parent = hash_node(current, sibling)`
/// `Right` → current is right child → `parent = hash_node(sibling, current)`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodePosition {
    Left,
    Right,
}

// ──────────────────────────────────────────────────────────────────────────────
// MerklePathNode
// ──────────────────────────────────────────────────────────────────────────────

/// One level in a Merkle authentication path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerklePathNode {
    /// The sibling's SHA-256 hash at this level.
    pub sibling: Digest,
    /// Which side the CURRENT node occupies at this level.
    pub position: NodePosition,
}

// ──────────────────────────────────────────────────────────────────────────────
// MerklePath
// ──────────────────────────────────────────────────────────────────────────────

/// An authentication path from a leaf to the Merkle root.
///
/// `nodes[0]` is closest to the leaf; `nodes[len-1]` is closest to the root.
/// Maximum length: `MAX_MERKLE_DEPTH` (40). Construction fails beyond this.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerklePath {
    pub nodes: Vec<MerklePathNode>,
}

impl MerklePath {
    /// Construct a path, enforcing the depth limit immediately.
    pub fn new(nodes: Vec<MerklePathNode>) -> Result<Self, TransitionError> {
        if nodes.len() > MAX_MERKLE_DEPTH {
            return Err(TransitionError::InvalidMerkleWitness);
        }
        Ok(Self { nodes })
    }

    /// Verify that walking this path from `leaf_hash` reaches `expected_root`.
    ///
    /// Returns `Err(InvalidMerkleWitness)` if the derived root does not match.
    /// This is the primary authentication step for CURRENT leaf state.
    pub fn verify(
        &self,
        leaf_hash: Digest,
        expected_root: Digest,
    ) -> Result<(), TransitionError> {
        if self.walk(leaf_hash) != expected_root {
            Err(TransitionError::InvalidMerkleWitness)
        } else {
            Ok(())
        }
    }

    /// Walk this path with a NEW leaf hash to derive the new root after mutation.
    ///
    /// Uses the same sibling set as `verify()` — the path structure is shared.
    /// The caller must have already called `verify(old_leaf_hash, current_root)`
    /// before calling this; `reconstruct_root` does not re-verify.
    pub fn reconstruct_root(&self, new_leaf_hash: Digest) -> Digest {
        self.walk(new_leaf_hash)
    }

    /// Internal: walk the path from `start` to the root using stored siblings.
    fn walk(&self, start: Digest) -> Digest {
        let mut current = start;
        for node in &self.nodes {
            current = match node.position {
                // Current is LEFT child: parent = hash_node(current, sibling)
                NodePosition::Left  => hash_node(&current, &node.sibling),
                // Current is RIGHT child: parent = hash_node(sibling, current)
                NodePosition::Right => hash_node(&node.sibling, &current),
            };
        }
        current
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// LeafMutation
// ──────────────────────────────────────────────────────────────────────────────

/// A single authenticated leaf update in a Merkle pool.
///
/// Field names and size limits from `witness_schema.md §Struct Layout`.
/// On-wire, `key` must be the canonical JCS-encoded identifier for this entry.
/// The kernel extracts the key field from `old_value` and asserts it matches
/// `LeafMutation.key` before accepting the path (Gap 1 invariant).
#[derive(Clone, Debug)]
pub struct LeafMutation {
    /// Canonical identifier for this leaf (JCS-encoded key, ≤ MAX_KEY_BYTES).
    /// For validator set: lowercase hex of Ed25519 public key.
    /// For impact pool: lowercase hex of SHA-256 of ProofOfImpact canonical bytes.
    /// For bond pool: lowercase hex of SHA-256 of VouchBond canonical bytes.
    pub key: Vec<u8>,

    /// Canonical bytes of the leaf value BEFORE this mutation.
    /// Empty (`[]`) means this is an INSERT (leaf did not previously exist).
    /// In that case: `hash_leaf([]) == empty_tree_root()` — both equal SHA256([0x00]).
    pub old_value: Vec<u8>,

    /// Canonical bytes of the leaf value AFTER this mutation.
    /// Empty (`[]`) means this is a DELETE (validator withdrawal only in v0.0.2).
    pub new_value: Vec<u8>,

    /// Authentication path for this leaf, relative to the EVOLVING pool root
    /// (Model A). The host constructs this path accounting for all prior
    /// mutations that have already been applied to this pool in this epoch.
    pub path: MerklePath,
}

impl LeafMutation {
    /// Validate all size constraints.
    /// Does NOT verify the Merkle path — call `path.verify()` separately.
    pub fn validate_sizes(&self) -> Result<(), TransitionError> {
        if self.key.is_empty() || self.key.len() > MAX_KEY_BYTES {
            return Err(TransitionError::InvalidSerialization);
        }
        if self.old_value.len() > MAX_VALUE_BYTES {
            return Err(TransitionError::InvalidSerialization);
        }
        if self.new_value.len() > MAX_VALUE_BYTES {
            return Err(TransitionError::InvalidSerialization);
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// EntropyStats
// ──────────────────────────────────────────────────────────────────────────────

/// Aggregate statistics for entropy metric computation.
///
/// Field names match `witness_schema.md §EntropyStats` and `state/entropy.rs`.
/// This is the ONLY acknowledged host-trust surface in v0.0.2 — the kernel
/// cannot independently verify `total_supply_raw` or `unique_active_validators`
/// without O(N) witnesses spanning the entire validator set.
///
/// The kernel verifies: `active_bonded_magnitude_raw ≤ total_supply_raw`
/// and `optimal_validator_count > 0`. All other values are host-trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntropyStats {
    /// Sum of all active VouchBond magnitudes this epoch (raw `Fixed` inner u128).
    pub active_bonded_magnitude_raw: u128,
    /// Total circulating supply at epoch start (raw `Fixed` inner u128).
    pub total_supply_raw: u128,
    /// Count of unique validators that submitted ≥ 1 payload this epoch.
    pub unique_active_validators: u64,
    /// Target validator set size from the Genesis Manifest (must be > 0).
    pub optimal_validator_count: u64,
}

impl EntropyStats {
    /// Validate the internally-checkable constraints.
    pub fn validate(&self) -> Result<(), TransitionError> {
        // Bonded amount cannot exceed total supply.
        if self.active_bonded_magnitude_raw > self.total_supply_raw {
            return Err(TransitionError::MathOverflow);
        }
        // Optimal count of zero would cause DivisionByZero in entropy computation.
        if self.optimal_validator_count == 0 {
            return Err(TransitionError::DivisionByZero);
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ValidatorSignature
// ──────────────────────────────────────────────────────────────────────────────

/// Maximum number of validator signatures per epoch.
/// Matches `MAX_PAYLOADS_PER_EPOCH` — no epoch can have more signers than payloads.
pub const MAX_VALIDATOR_SIGNATURES: usize = MAX_PAYLOADS_PER_EPOCH;

/// A single Ed25519 signature from a validator authorizing this epoch transition.
///
/// Within `StateWitnessBundle.validator_signatures`, entries MUST be in strictly
/// ascending order of `validator_pubkey`. No duplicate pubkeys are permitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorSignature {
    /// Ed25519 public key (32 bytes, compressed Edwards y-coordinate + sign bit).
    pub validator_pubkey: [u8; 32],
    /// Ed25519 signature (64 bytes: R || s).
    pub signature: [u8; 64],
}

// ──────────────────────────────────────────────────────────────────────────────
// StateWitnessBundle
// ──────────────────────────────────────────────────────────────────────────────

/// Everything the host provides for one epoch transition.
///
/// Field names are alphabetical (JCS canonical order for eventual serialization)
/// and match the three Merkle pool roots in `EpochState`:
/// `bond_pool_root`, `impact_pool_root`, `validator_set_root`.
///
/// Within each `Vec<LeafMutation>`, entries MUST be in strictly ascending
/// lexicographic order of `key`. The kernel rejects out-of-order witnesses.
/// No key may appear in more than one pool's array.
#[derive(Clone, Debug)]
pub struct StateWitnessBundle {
    /// Witness mutations for the bond pool tree (`EpochState.bond_pool_root`).
    pub bond_witnesses: Vec<LeafMutation>,
    /// Aggregate entropy statistics (partially host-trusted — see EntropyStats).
    pub entropy_stats: EntropyStats,
    /// Witness mutations for the impact pool tree (`EpochState.impact_pool_root`).
    pub impact_witnesses: Vec<LeafMutation>,
    /// Ed25519 signatures authorizing this epoch transition.
    /// Strictly ascending pubkey order, no duplicates.
    /// HOST-TRUSTED (v0.0.2): Pubkeys are NOT verified against validator_set_root.
    /// Full Merkle membership proofs required in v0.0.3.
    pub validator_signatures: Vec<ValidatorSignature>,
    /// Witness mutations for the validator set tree (`EpochState.validator_set_root`).
    /// Processed in two passes: registration first, then decay.
    pub validator_witnesses: Vec<LeafMutation>,
}

impl StateWitnessBundle {
    /// Validate the combined payload count against `MAX_PAYLOADS_PER_EPOCH`.
    /// Called before any Merkle verification — reject oversized bundles immediately.
    pub fn validate_limits(&self) -> Result<(), TransitionError> {
        let total = self.bond_witnesses.len()
            .saturating_add(self.impact_witnesses.len())
            .saturating_add(self.validator_witnesses.len());
        if total > MAX_PAYLOADS_PER_EPOCH {
            return Err(TransitionError::PayloadLimitExceeded);
        }
        if self.validator_signatures.len() > MAX_VALIDATOR_SIGNATURES {
            return Err(TransitionError::PayloadLimitExceeded);
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Signature Gate Functions
// ──────────────────────────────────────────────────────────────────────────────

/// Domain separation prefix for epoch signing root (distinct from leaf=0x00, node=0x01).
const SIGNING_DOMAIN_PREFIX: u8 = 0x02;

/// Compute the canonical hash of all three mutation vectors.
///
/// Format (frozen — any change forks the protocol):
/// ```text
/// len(bond_witnesses)_be4 || bond_bytes ||
/// len(impact_witnesses)_be4 || impact_bytes ||
/// len(validator_witnesses)_be4 || validator_bytes
/// ```
///
/// Where each mutation is serialized as:
/// ```text
/// len(key)_be2 || key || len(old_value)_be2 || old_value || len(new_value)_be2 || new_value
/// ```
///
/// Path data is NOT included — paths are structural, not content.
pub fn compute_bundle_hash(witness: &StateWitnessBundle) -> Digest {
    let mut buf = Vec::new();
    serialize_mutations(&mut buf, &witness.bond_witnesses);
    serialize_mutations(&mut buf, &witness.impact_witnesses);
    serialize_mutations(&mut buf, &witness.validator_witnesses);
    sha256(&buf)
}

/// Serialize a mutation vector in canonical format.
fn serialize_mutations(buf: &mut Vec<u8>, mutations: &[LeafMutation]) {
    // 4-byte big-endian count (max 10,000 fits in u32).
    buf.extend_from_slice(&(mutations.len() as u32).to_be_bytes());
    for m in mutations {
        // key: 2-byte len + bytes
        buf.extend_from_slice(&(m.key.len() as u16).to_be_bytes());
        buf.extend_from_slice(&m.key);
        // old_value: 2-byte len + bytes
        buf.extend_from_slice(&(m.old_value.len() as u16).to_be_bytes());
        buf.extend_from_slice(&m.old_value);
        // new_value: 2-byte len + bytes
        buf.extend_from_slice(&(m.new_value.len() as u16).to_be_bytes());
        buf.extend_from_slice(&m.new_value);
    }
}

/// Compute the epoch signing root — the digest that validators sign.
///
/// ```text
/// SHA256(0x02 || prev_state_root || bundle_hash || epoch_number_be8 || kernel_hash)
/// ```
///
/// - `0x02`: domain separation (leaf=0x00, node=0x01, signing=0x02)
/// - `prev_state_root`: binds history
/// - `bundle_hash`: binds all witness content
/// - `epoch_number_be8`: prevents replay
/// - `kernel_hash`: binds protocol version
///
/// Total input: 1 + 32 + 32 + 8 + 32 = 105 bytes.
pub fn compute_epoch_signing_root(
    prev_state_root: &Digest,
    bundle_hash: &Digest,
    epoch_number: u64,
    kernel_hash: &Digest,
) -> Digest {
    let mut buf = [0u8; 105];
    buf[0] = SIGNING_DOMAIN_PREFIX;
    buf[1..33].copy_from_slice(prev_state_root);
    buf[33..65].copy_from_slice(bundle_hash);
    buf[65..73].copy_from_slice(&epoch_number.to_be_bytes());
    buf[73..105].copy_from_slice(kernel_hash);
    sha256(&buf)
}

/// Verify quorum: structural checks + cryptographic verification.
///
/// Enforces:
/// 1. Strict ascending pubkey order (no duplicates)
/// 2. All signatures verify against `signing_root` via `verify_strict`
/// 3. Count ≥ ⌈2/3 × optimal_validator_count⌉
///
/// HOST-TRUSTED (v0.0.2): Pubkeys are NOT verified against validator_set_root.
/// Full Merkle membership proofs required in v0.0.3.
///
/// All signatures are verified before checking threshold — no early exit.
/// This prevents adversaries from manipulating timing behavior.
pub fn verify_quorum(
    signatures: &[ValidatorSignature],
    signing_root: &Digest,
    optimal_validator_count: u64,
) -> Result<(), TransitionError> {
    use crate::physics::ed25519;

    // ── Step 1: Structural checks ──────────────────────────────────────────
    // Strict ascending pubkey order, no duplicates.
    for i in 1..signatures.len() {
        if signatures[i].validator_pubkey <= signatures[i - 1].validator_pubkey {
            // Duplicate or reversed order.
            return Err(TransitionError::InvalidSerialization);
        }
    }

    // ── Step 2: Cryptographic verification ──────────────────────────────────
    // Verify ALL signatures before checking threshold.
    // No early exit — constant-time traversal prevents timing attacks.
    for sig in signatures {
        ed25519::verify(&sig.validator_pubkey, signing_root, &sig.signature)?;
    }

    // ── Step 3: Threshold check ────────────────────────────────────────────
    // threshold = ceil(2/3 * n) = (2*n + 2) / 3  (integer math)
    // Special case: if optimal_validator_count == 0, threshold == 0,
    // and empty signatures is valid (genesis or no-validator epoch).
    let threshold = (2 * optimal_validator_count + 2) / 3;
    if (signatures.len() as u64) < threshold {
        return Err(TransitionError::InvalidSignature);
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────
// apply_pool_mutations — Core State Transition Function
// ──────────────────────────────────────────────────────────────────────────────

/// Apply a sequence of authenticated leaf mutations to a Merkle pool root.
///
/// This is the constitutional bridge between witness types and state transition.
/// It enforces **Model A (evolving-root verification)**: each mutation's path
/// is verified against the root produced by the preceding mutation, not the
/// original pool root.
///
/// # Constitutional Rules Enforced
///
/// 1. Mutations must be in **strictly ascending lexicographic key order**.
///    Equal keys (duplicates) and reversed keys are both rejected.
/// 2. Each mutation's path is verified against the **current intermediate root**,
///    not `prev_state.<pool>_root`. The root evolves with every mutation.
/// 3. The **final returned root** is the root reconstructed after the last mutation.
///    The caller writes this into the new `EpochState`.
/// 4. An empty mutation list is valid: returns `current_root` unchanged.
///    This is the empty-epoch passthrough for pools with no activity.
///
/// # Errors
///
/// - `InvalidSerialization` — mutations are out of lexicographic key order,
///   or contain duplicate keys.
/// - `InvalidMerkleWitness` — any mutation's path does not verify against
///   the current intermediate root.
pub fn apply_pool_mutations(
    current_root: Digest,
    mutations: &[LeafMutation],
) -> Result<Digest, TransitionError> {
    // ── Step 1: Empty fast path ───────────────────────────────────────────────
    // No mutations → root is unchanged. Valid for pools with no epoch activity.
    if mutations.is_empty() {
        return Ok(current_root);
    }

    // ── Step 2: Enforce strictly ascending key ordering ───────────────────────
    // Keys must be strictly increasing (no duplicates, no reversal).
    // This rule is from witness_schema.md §Witness Validity Invariants (4).
    for i in 1..mutations.len() {
        if mutations[i - 1].key >= mutations[i].key {
            return Err(TransitionError::InvalidSerialization);
        }
    }

    // ── Step 3: Evolving-root verification loop (Model A) ─────────────────────
    let mut intermediate_root = current_root;

    for mutation in mutations {
        // 3a. Compute old leaf hash.
        //     hash_leaf([]) == empty_tree_root() for INSERT case — correct by spec.
        let old_leaf_hash = hash_leaf(&mutation.old_value);

        // 3b. Verify the path against the CURRENT intermediate root, not the
        //     original pool root. This enforces Model A: stale paths from
        //     before a prior mutation fail here.
        mutation.path.verify(old_leaf_hash, intermediate_root)?;

        // 3c. Reconstruct the new intermediate root using the new leaf value.
        let new_leaf_hash = hash_leaf(&mutation.new_value);
        intermediate_root = mutation.path.reconstruct_root(new_leaf_hash);
    }

    // ── Step 4: Return the final root ─────────────────────────────────────────
    // This is written directly into EpochState.<pool>_root by the caller.
    Ok(intermediate_root)
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::hashing::{hash_leaf, hash_node};
    use crate::physics::merkle::empty_tree_root;

    // ── Position semantics ────────────────────────────────────────────────────

    #[test]
    fn left_position_means_current_is_left_child() {
        // Tree:  root
        //        /  \
        //    leaf    sibling
        // parent = hash_node(leaf, sibling)  [leaf is LEFT child]
        let leaf    = hash_leaf(b"a");
        let sibling = hash_leaf(b"b");
        let expected_root = hash_node(&leaf, &sibling);

        let path = MerklePath::new(vec![MerklePathNode {
            sibling,
            position: NodePosition::Left, // current (leaf) is LEFT
        }]).unwrap();

        path.verify(leaf, expected_root).unwrap();
    }

    #[test]
    fn right_position_means_current_is_right_child() {
        // Tree:  root
        //        /  \
        //    sibling  leaf
        // parent = hash_node(sibling, leaf)  [leaf is RIGHT child]
        let sibling = hash_leaf(b"a");
        let leaf    = hash_leaf(b"b");
        let expected_root = hash_node(&sibling, &leaf);

        let path = MerklePath::new(vec![MerklePathNode {
            sibling,
            position: NodePosition::Right, // current (leaf) is RIGHT
        }]).unwrap();

        path.verify(leaf, expected_root).unwrap();
    }

    // ── Empty path (single-leaf tree) ─────────────────────────────────────────

    #[test]
    fn empty_path_verifies_single_leaf_tree() {
        // A tree with exactly one leaf: root == hash_leaf(value).
        // No siblings exist, so path is empty.
        let leaf_value = b"single";
        let leaf_hash  = hash_leaf(leaf_value);

        let path = MerklePath::new(vec![]).unwrap();
        path.verify(leaf_hash, leaf_hash).unwrap();
    }

    // ── reconstruct_root ──────────────────────────────────────────────────────

    #[test]
    fn reconstruct_root_produces_new_root_after_mutation() {
        // Tree:  root
        //        /  \
        //       A    B
        let leaf_a = hash_leaf(b"a");
        let leaf_b = hash_leaf(b"b");
        let root   = hash_node(&leaf_a, &leaf_b);

        let path = MerklePath::new(vec![MerklePathNode {
            sibling:  leaf_b,
            position: NodePosition::Left, // A is the left child
        }]).unwrap();

        // Verify A is in tree at root.
        path.verify(leaf_a, root).unwrap();

        // Mutate: replace A with A2.
        let leaf_a2   = hash_leaf(b"a2");
        let new_root  = path.reconstruct_root(leaf_a2);
        let expected  = hash_node(&leaf_a2, &leaf_b);
        assert_eq!(new_root, expected);
    }

    // ── Wrong root rejected ───────────────────────────────────────────────────

    #[test]
    fn wrong_expected_root_is_rejected() {
        let leaf = hash_leaf(b"x");
        let path = MerklePath::new(vec![]).unwrap();
        assert_eq!(
            path.verify(leaf, [0u8; 32]),
            Err(TransitionError::InvalidMerkleWitness)
        );
    }

    #[test]
    fn wrong_sibling_produces_different_root() {
        let leaf    = hash_leaf(b"a");
        let sibling = hash_leaf(b"b");
        let root    = hash_node(&leaf, &sibling);

        // Path with wrong sibling.
        let bad_path = MerklePath::new(vec![MerklePathNode {
            sibling:  hash_leaf(b"WRONG"),
            position: NodePosition::Left,
        }]).unwrap();

        assert_eq!(
            bad_path.verify(leaf, root),
            Err(TransitionError::InvalidMerkleWitness)
        );
    }

    // ── Depth limit ───────────────────────────────────────────────────────────

    #[test]
    fn path_at_depth_limit_is_accepted() {
        let nodes = vec![
            MerklePathNode { sibling: [0u8; 32], position: NodePosition::Left };
            MAX_MERKLE_DEPTH
        ];
        assert!(MerklePath::new(nodes).is_ok());
    }

    #[test]
    fn path_exceeding_depth_limit_is_rejected() {
        let nodes = vec![
            MerklePathNode { sibling: [0u8; 32], position: NodePosition::Left };
            MAX_MERKLE_DEPTH + 1
        ];
        assert_eq!(
            MerklePath::new(nodes),
            Err(TransitionError::InvalidMerkleWitness)
        );
    }

    // ── Empty leaf identity (constitutional) ──────────────────────────────────

    #[test]
    fn hash_leaf_empty_equals_empty_tree_root() {
        // CONSTITUTIONAL: hash_leaf([]) == empty_tree_root()
        // Both = SHA256([0x00]). Frozen by witness_schema.md.
        // Breaking this identity is a fork.
        assert_eq!(hash_leaf(b""), empty_tree_root(),
            "hash_leaf([]) must equal empty_tree_root() — both are SHA256([0x00])");
    }

    // ── EntropyStats validation ───────────────────────────────────────────────

    #[test]
    fn entropy_stats_rejects_bonded_exceeding_supply() {
        let bad = EntropyStats {
            active_bonded_magnitude_raw: 1001,
            total_supply_raw: 1000,
            unique_active_validators: 10,
            optimal_validator_count: 100,
        };
        assert_eq!(bad.validate(), Err(TransitionError::MathOverflow));
    }

    #[test]
    fn entropy_stats_rejects_zero_optimal_count() {
        let bad = EntropyStats {
            active_bonded_magnitude_raw: 0,
            total_supply_raw: 1000,
            unique_active_validators: 10,
            optimal_validator_count: 0,
        };
        assert_eq!(bad.validate(), Err(TransitionError::DivisionByZero));
    }

    #[test]
    fn entropy_stats_accepts_bonded_equal_to_supply() {
        let ok = EntropyStats {
            active_bonded_magnitude_raw: 1000,
            total_supply_raw: 1000,
            unique_active_validators: 10,
            optimal_validator_count: 100,
        };
        assert!(ok.validate().is_ok());
    }

    // ── StateWitnessBundle payload limit ──────────────────────────────────────

    #[test]
    fn bundle_over_payload_limit_is_rejected() {
        let dummy_mutation = LeafMutation {
            key: b"k".to_vec(),
            old_value: vec![],
            new_value: b"v".to_vec(),
            path: MerklePath::new(vec![]).unwrap(),
        };
        // MAX_PAYLOADS_PER_EPOCH + 1 total across all pools.
        let bundle = StateWitnessBundle {
            bond_witnesses: vec![dummy_mutation.clone(); MAX_PAYLOADS_PER_EPOCH / 2 + 1],
            entropy_stats: EntropyStats {
                active_bonded_magnitude_raw: 0,
                total_supply_raw: 1,
                unique_active_validators: 1,
                optimal_validator_count: 1,
            },
            impact_witnesses: vec![dummy_mutation; MAX_PAYLOADS_PER_EPOCH / 2 + 1],
            validator_signatures: vec![],
            validator_witnesses: vec![],
        };
        assert_eq!(bundle.validate_limits(), Err(TransitionError::PayloadLimitExceeded));
    }

    // ── LeafMutation size validation ──────────────────────────────────────────

    #[test]
    fn leaf_mutation_rejects_empty_key() {
        let m = LeafMutation {
            key: vec![],
            old_value: vec![],
            new_value: vec![],
            path: MerklePath::new(vec![]).unwrap(),
        };
        assert_eq!(m.validate_sizes(), Err(TransitionError::InvalidSerialization));
    }

    #[test]
    fn leaf_mutation_rejects_oversized_value() {
        let m = LeafMutation {
            key: b"k".to_vec(),
            old_value: vec![0u8; MAX_VALUE_BYTES + 1],
            new_value: vec![],
            path: MerklePath::new(vec![]).unwrap(),
        };
        assert_eq!(m.validate_sizes(), Err(TransitionError::InvalidSerialization));
    }

    // ── Pinned constitutional vector ──────────────────────────────────────────

    #[test]
    fn two_leaf_mutation_verify_and_reconstruct_is_pinned() {
        // CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        //
        // Tree (two leaves):
        //     root = hash_node(hash_leaf(b"a"), hash_leaf(b"b"))
        //
        // Mutation: replace leaf "a" with "a2".
        // New root = hash_node(hash_leaf(b"a2"), hash_leaf(b"b"))
        //
        // Leaf "a" is the LEFT child (position = Left).
        // Leaf "b" is the sibling on the RIGHT.
        let leaf_a  = hash_leaf(b"a");
        let leaf_b  = hash_leaf(b"b");
        let old_root = hash_node(&leaf_a, &leaf_b);

        let path = MerklePath::new(vec![MerklePathNode {
            sibling:  leaf_b,
            position: NodePosition::Left,
        }]).unwrap();

        // Verify old leaf sits in old root.
        path.verify(leaf_a, old_root).unwrap();

        // Reconstruct new root after mutation.
        let leaf_a2  = hash_leaf(b"a2");
        let new_root = path.reconstruct_root(leaf_a2);
        let expected = hash_node(&leaf_a2, &leaf_b);
        assert_eq!(new_root, expected,
            "two-leaf mutation must produce the correct new root");

        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // old_root = hash_node(hash_leaf("a"), hash_leaf("b"))
        // new_root = hash_node(hash_leaf("a2"), hash_leaf("b"))
        // Any change to hash_leaf, hash_node, or NodePosition semantics breaks this.
        let expected_old_root: [u8; 32] = [
            0xb1, 0x37, 0x98, 0x5f, 0xf4, 0x84, 0xfb, 0x60,
            0x0d, 0xb9, 0x31, 0x07, 0xc7, 0x7b, 0x03, 0x65,
            0xc8, 0x0d, 0x78, 0xf5, 0xb4, 0x29, 0xde, 0xd0,
            0xfd, 0x97, 0x36, 0x1d, 0x07, 0x79, 0x99, 0xeb,
        ];
        let expected_new_root: [u8; 32] = [
            0xce, 0x09, 0x3f, 0x77, 0xc5, 0x46, 0x7d, 0x40,
            0x5c, 0x9e, 0xe9, 0xdb, 0xbd, 0xd8, 0x07, 0x85,
            0x02, 0x99, 0x3e, 0x9b, 0x6f, 0xc8, 0x47, 0x6e,
            0x31, 0xed, 0x7c, 0x69, 0x57, 0xcd, 0xaf, 0xcb,
        ];
        assert_eq!(old_root, expected_old_root, "old_root diverged — hash_leaf or hash_node changed");
        assert_eq!(new_root, expected_new_root, "new_root diverged — Merkle mutation semantics changed");
    }

    // ── apply_pool_mutations ──────────────────────────────────────────────────

    /// Build a single-level LeafMutation for a two-leaf tree.
    /// Leaf is the LEFT child (key "a"), sibling is the RIGHT child (key "b").
    fn make_mutation(
        key: &[u8],
        old_value: &[u8],
        new_value: &[u8],
        sibling: Digest,
        position: NodePosition,
    ) -> LeafMutation {
        LeafMutation {
            key: key.to_vec(),
            old_value: old_value.to_vec(),
            new_value: new_value.to_vec(),
            path: MerklePath::new(vec![MerklePathNode { sibling, position }]).unwrap(),
        }
    }

    #[test]
    fn empty_mutations_returns_root_unchanged() {
        let root = hash_node(&hash_leaf(b"a"), &hash_leaf(b"b"));
        let result = apply_pool_mutations(root, &[]).unwrap();
        assert_eq!(result, root, "empty mutation list must not change the root");
    }

    #[test]
    fn single_mutation_produces_correct_new_root() {
        // Tree: root = hash_node(A, B). Mutate A → A2.
        let leaf_a = hash_leaf(b"a");
        let leaf_b = hash_leaf(b"b");
        let root = hash_node(&leaf_a, &leaf_b);

        let mutations = vec![make_mutation(
            b"a", b"a", b"a2", leaf_b, NodePosition::Left,
        )];

        let new_root = apply_pool_mutations(root, &mutations).unwrap();
        let expected = hash_node(&hash_leaf(b"a2"), &leaf_b);
        assert_eq!(new_root, expected);
    }

    #[test]
    fn two_sequential_mutations_use_evolving_root_model_a() {
        // Tree: root = hash_node(A, B). Apply two mutations in order:
        //   1) A → A2  (key "a")
        //   2) B → B2  (key "b"), path relative to intermediate root after mutation 1.
        let leaf_a  = hash_leaf(b"a");
        let leaf_b  = hash_leaf(b"b");
        let leaf_a2 = hash_leaf(b"a2");
        let leaf_b2 = hash_leaf(b"b2");

        let original_root = hash_node(&leaf_a, &leaf_b);
        // After mutation 1: intermediate = hash_node(A2, B)
        let intermediate  = hash_node(&leaf_a2, &leaf_b);

        // Mutation 1: A → A2, path relative to original_root.
        let m1 = make_mutation(b"a", b"a", b"a2", leaf_b, NodePosition::Left);
        // Mutation 2: B → B2, path relative to intermediate (Model A).
        let m2 = make_mutation(b"b", b"b", b"b2", leaf_a2, NodePosition::Right);

        let final_root = apply_pool_mutations(original_root, &[m1, m2]).unwrap();
        let expected   = hash_node(&leaf_a2, &leaf_b2);
        assert_eq!(final_root, expected,
            "two sequential mutations must produce hash_node(A2, B2)");

        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // Tree hash_node(A, B). Apply A→A2 then B→B2 via Model A evolving root.
        // Final root = hash_node(hash_leaf("a2"), hash_leaf("b2"))
        // = 079161dd45f4653477aac13c77f7a034300c61f3fb8627ebecdee87d86f83018
        // Any change to apply_pool_mutations, hash_leaf, hash_node, or
        // NodePosition semantics will break this assertion immediately.
        let expected_final_root: [u8; 32] = [
            0x07, 0x91, 0x61, 0xdd, 0x45, 0xf4, 0x65, 0x34,
            0x77, 0xaa, 0xc1, 0x3c, 0x77, 0xf7, 0xa0, 0x34,
            0x30, 0x0c, 0x61, 0xf3, 0xfb, 0x86, 0x27, 0xeb,
            0xec, 0xde, 0xe8, 0x7d, 0x86, 0xf8, 0x30, 0x18,
        ];
        assert_eq!(final_root, expected_final_root,
            "two-mutation final root diverged — apply_pool_mutations execution path changed");
    }

    #[test]
    fn duplicate_key_is_rejected() {
        let leaf_a = hash_leaf(b"a");
        let leaf_b = hash_leaf(b"b");
        let root   = hash_node(&leaf_a, &leaf_b);

        // Same key "a" twice — must be rejected.
        let m1 = make_mutation(b"a", b"a", b"a2", leaf_b, NodePosition::Left);
        let m2 = make_mutation(b"a", b"a2", b"a3", leaf_b, NodePosition::Left);

        assert_eq!(
            apply_pool_mutations(root, &[m1, m2]),
            Err(TransitionError::InvalidSerialization),
            "duplicate key must be rejected"
        );
    }

    #[test]
    fn reversed_key_order_is_rejected() {
        let leaf_a = hash_leaf(b"a");
        let leaf_b = hash_leaf(b"b");
        let root   = hash_node(&leaf_a, &leaf_b);

        // Correct mutations but submitted in wrong order (b before a).
        let m_b = make_mutation(b"b", b"b", b"b2", leaf_a, NodePosition::Right);
        let m_a = make_mutation(b"a", b"a", b"a2", leaf_b, NodePosition::Left);

        assert_eq!(
            apply_pool_mutations(root, &[m_b, m_a]),
            Err(TransitionError::InvalidSerialization),
            "reversed key order must be rejected"
        );
    }

    #[test]
    fn stale_path_fails_on_second_mutation_model_a_enforced() {
        // Tree: root = hash_node(A, B).
        // Both mutations have paths relative to the ORIGINAL root (Model B style).
        // The second mutation must fail because its path is stale after mutation 1.
        let leaf_a  = hash_leaf(b"a");
        let leaf_b  = hash_leaf(b"b");
        let root    = hash_node(&leaf_a, &leaf_b);

        // Both paths reference the original sibling (stale after mutation 1).
        let m1 = make_mutation(b"a", b"a", b"a2", leaf_b, NodePosition::Left);
        // m2's path sibling is still leaf_a (original), but after m1, the tree
        // has leaf_a2 on the left — so the reconstructed root from m1 will differ.
        let m2 = make_mutation(b"b", b"b", b"b2", leaf_a, NodePosition::Right);

        // m2 must fail: its path (sibling = leaf_a) verifies against
        // hash_node(leaf_a2, leaf_b), not hash_node(leaf_a, leaf_b).
        assert_eq!(
            apply_pool_mutations(root, &[m1, m2]),
            Err(TransitionError::InvalidMerkleWitness),
            "stale path from before a prior mutation must fail (Model A enforced)"
        );
    }
}
