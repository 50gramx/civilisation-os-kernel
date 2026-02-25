//! `EpochState` — the self-committing canonical state root.
//!
//! # What This Is
//!
//! `EpochState` is the only thing the consensus layer needs to agree on.
//! It is a flat set of 8 fixed-width fields, all `[u8; 32]` or `u128`.
//! There are no generics, no trait bounds, no heap allocation, no Vec.
//! The struct is fully stack-allocated and copy-friendly.
//!
//! # Canonical Serialization (Frozen)
//!
//! The `state_root` is computed as:
//!
//! ```text
//! state_root = SHA256(canonicalize(EpochState_without_state_root))
//! ```
//!
//! `canonicalize()` produces RFC 8785 JSON with keys in lexicographic byte order.
//! The physical field ordering in the serialized object is FROZEN as:
//!
//! 1. `bond_pool_root`         — hex string (64 chars)
//! 2. `entropy_metric_scaled`  — decimal u128 string (raw Fixed inner value)
//! 3. `epoch_number`           — decimal u64 string
//! 4. `impact_pool_root`       — hex string (64 chars)
//! 5. `kernel_hash`            — hex string (64 chars)
//! 6. `previous_root`          — hex string (64 chars)
//! 7. `validator_set_root`     — hex string (64 chars)
//! 8. `vdf_challenge_seed`     — hex string (64 chars)
//!
//! This ordering is alphabetical by key name, which is what `canonicalize()` enforces.
//! It is documented here explicitly so that it survives future code refactors.
//!
//! # Invariants
//!
//! - `entropy_metric_scaled` stores the **raw u128** inner value of `Fixed`, NOT a
//!   floating-point representation. The scale factor `SCALE = 10^12` is implicit.
//!   Changing the scale or representation format is a hard fork.
//!
//! - `kernel_hash` is the SHA-256 of the WASM kernel binary that produced this state.
//!   It prevents cross-kernel fraud proof replay and detects silent binary upgrades.
//!
//! - `previous_root` chains this epoch to the one before it.
//!   The thermodynamic arrow of time is cryptographically enforced.
//!
//! - `state_root` is excluded from its own canonical serialization to avoid
//!   the circular dependency. It is always the LAST field to be computed.

use crate::math::fixed::Fixed;
use crate::physics::hashing::{sha256, Digest};
use crate::physics::canonical_json::canonicalize;
use crate::TransitionError;

// ──────────────────────────────────────────────────────────────────────────────
// Constitutional constants
// ──────────────────────────────────────────────────────────────────────────────

/// Maximum payloads (ProofOfImpact + VouchBond combined) accepted per epoch.
pub const MAX_PAYLOADS_PER_EPOCH: usize = 10_000;

/// Fraud proofs older than this many epochs are permanently rejected.
pub const MAX_FRAUD_WINDOW_EPOCHS: u64 = 1;

// ──────────────────────────────────────────────────────────────────────────────
// Struct definition
// ──────────────────────────────────────────────────────────────────────────────

/// The canonical committed state at the end of one epoch.
///
/// **No generics. No trait bounds. No heap allocation. Pure physics.**
///
/// Only Merkle roots are stored — not the full materialized state.
/// This keeps the struct small enough to fit in WASM memory budgets
/// regardless of how many identities exist in the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochState {
    // ── Serialized fields (alphabetically, all included in state_root) ─────

    /// Merkle root committing to the set of active `VouchBond` locks.
    pub bond_pool_root: Digest,

    /// Global entropy metric for this epoch.
    /// Stored as the **raw u128 inner value** of `crate::math::fixed::Fixed`.
    /// Scale factor is `SCALE = 10^12`. Representation is immutable.
    pub entropy_metric_scaled: u128,

    /// Monotonically increasing epoch counter. Genesis is 0.
    pub epoch_number: u64,

    /// Merkle root committing to all validated `ProofOfImpact` records.
    pub impact_pool_root: Digest,

    /// SHA-256 of the WASM kernel binary that produced this state.
    /// Binds on-chain commitments to a specific auditable kernel version.
    /// Prevents cross-kernel fraud proof replay attacks.
    pub kernel_hash: Digest,

    /// `state_root` of the immediately preceding epoch.
    /// The chain of `previous_root` hashes is the thermodynamic arrow of time.
    pub previous_root: Digest,

    // ── Self-committing hash (NOT included in its own serialization) ───────

    /// SHA-256 of the canonical serialization of all other fields.
    /// Computed last. Excluded from the canonical bytes it hashes over.
    pub state_root: Digest,

    /// Merkle root committing to the active validator set and stake weights.
    pub validator_set_root: Digest,

    /// VDF challenge seed used to derive the NEXT epoch's sortition randomness.
    /// Prevents look-ahead attacks — the seed is only known when this epoch closes.
    pub vdf_challenge_seed: Digest,
}

// ──────────────────────────────────────────────────────────────────────────────
// Serialization helpers (no external dependencies)
// ──────────────────────────────────────────────────────────────────────────────

const HEX: [u8; 16] = *b"0123456789abcdef";

/// Encode a 32-byte digest as 64 lowercase hex characters.
fn encode_digest(d: &Digest) -> Vec<u8> {
    let mut s = Vec::with_capacity(64);
    for &b in d.iter() {
        s.push(HEX[(b >> 4) as usize]);
        s.push(HEX[(b & 0xF) as usize]);
    }
    s
}

/// Encode a u128 as a decimal string (no leading zeros, no sign, no decimal).
fn encode_u128(n: u128) -> Vec<u8> {
    if n == 0 {
        return b"0".to_vec();
    }
    let mut digits = Vec::new();
    let mut v = n;
    while v > 0 {
        digits.push(b'0' + (v % 10) as u8);
        v /= 10;
    }
    digits.reverse();
    digits
}

/// Encode a u64 as a decimal string (no leading zeros, no sign).
fn encode_u64(n: u64) -> Vec<u8> {
    encode_u128(n as u128)
}

// ──────────────────────────────────────────────────────────────────────────────
// Canonical JSON builder
// ──────────────────────────────────────────────────────────────────────────────

/// Build the canonical JSON bytes for the 8 fields that contribute to `state_root`.
/// Fields are emitted in alphabetical order (matching what `canonicalize()` enforces).
/// The `state_root` field is deliberately excluded.
fn build_commitment_json(s: &EpochState) -> Vec<u8> {
    let mut out = Vec::with_capacity(512);

    // Emit a pre-sorted JSON object. All keys are lowercase ASCII so canonicalize()
    // will not reorder them — but we run through it anyway as a constitutional check.
    out.extend_from_slice(b"{\"bond_pool_root\":\"");
    out.extend_from_slice(&encode_digest(&s.bond_pool_root));
    out.extend_from_slice(b"\",\"entropy_metric_scaled\":\"");
    out.extend_from_slice(&encode_u128(s.entropy_metric_scaled));
    out.extend_from_slice(b"\",\"epoch_number\":\"");
    out.extend_from_slice(&encode_u64(s.epoch_number));
    out.extend_from_slice(b"\",\"impact_pool_root\":\"");
    out.extend_from_slice(&encode_digest(&s.impact_pool_root));
    out.extend_from_slice(b"\",\"kernel_hash\":\"");
    out.extend_from_slice(&encode_digest(&s.kernel_hash));
    out.extend_from_slice(b"\",\"previous_root\":\"");
    out.extend_from_slice(&encode_digest(&s.previous_root));
    out.extend_from_slice(b"\",\"validator_set_root\":\"");
    out.extend_from_slice(&encode_digest(&s.validator_set_root));
    out.extend_from_slice(b"\",\"vdf_challenge_seed\":\"");
    out.extend_from_slice(&encode_digest(&s.vdf_challenge_seed));
    out.extend_from_slice(b"\"}");

    out
}

// ──────────────────────────────────────────────────────────────────────────────
// impl EpochState
// ──────────────────────────────────────────────────────────────────────────────

impl EpochState {
    /// Returns the placeholder genesis state (epoch 0, all-zero roots).
    ///
    /// In production this is replaced by a Genesis Manifest signed by the
    /// founding committee. All-zero roots are valid placeholders for alpha testing.
    pub fn genesis() -> Self {
        let mut s = EpochState {
            bond_pool_root:        [0u8; 32],
            entropy_metric_scaled: 0,
            epoch_number:          0,
            impact_pool_root:      [0u8; 32],
            kernel_hash:           [0u8; 32],
            previous_root:         [0u8; 32],
            state_root:            [0u8; 32],
            validator_set_root:    [0u8; 32],
            vdf_challenge_seed:    [0u8; 32],
        };
        // Compute and assign the actual genesis state_root.
        s.state_root = s.compute_state_root().unwrap_or([0u8; 32]);
        s
    }

    /// Return the `entropy_metric_scaled` field as a typed `Fixed(u128)`.
    ///
    /// Returns `MathOverflow` if the stored raw value somehow exceeds
    /// `MAX_SAFE_BALANCE_RAW` (a kernel invariant violation, never expected).
    pub fn entropy(&self) -> Result<Fixed, TransitionError> {
        Fixed::from_raw(self.entropy_metric_scaled)
    }

    /// Produce the canonical JSON bytes that commit to this state.
    ///
    /// Runs through `canonicalize()` as a constitutional sanity check —
    /// if the generated JSON is not valid JCS, this is a kernel bug.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, TransitionError> {
        let raw = build_commitment_json(self);
        // Sanity check: the JSON we built must round-trip cleanly.
        // If canonicalize() produces different bytes, the field ordering is wrong.
        let checked = canonicalize(&raw)?;
        if checked != raw {
            // Our pre-sorted emission diverged from the canonicalizer output.
            // This is a kernel bug, not a user error.
            return Err(TransitionError::InvalidSerialization);
        }
        Ok(raw)
    }

    /// Compute the `state_root` from the current field values.
    ///
    /// Returns `SHA256(canonical_bytes(self))`.
    /// Does NOT mutate `self`; use `commit()` to assign the result.
    pub fn compute_state_root(&self) -> Result<Digest, TransitionError> {
        let bytes = self.canonical_bytes()?;
        Ok(sha256(&bytes))
    }

    /// Assign the computed `state_root` and return `self`.
    ///
    /// Call this as the LAST step of state construction, after all other fields
    /// are set. Returns `Err` if canonical serialization fails (kernel bug).
    pub fn commit(mut self) -> Result<Self, TransitionError> {
        self.state_root = self.compute_state_root()?;
        Ok(self)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // ── Serialization correctness ─────────────────────────────────────────────

    #[test]
    fn all_zero_genesis_canonical_bytes_are_stable() {
        // The all-zero genesis produces a known canonical JSON byte sequence.
        let s = EpochState {
            bond_pool_root:        [0u8; 32],
            entropy_metric_scaled: 0,
            epoch_number:          0,
            impact_pool_root:      [0u8; 32],
            kernel_hash:           [0u8; 32],
            previous_root:         [0u8; 32],
            state_root:            [0u8; 32],  // excluded from its own serialization
            validator_set_root:    [0u8; 32],
            vdf_challenge_seed:    [0u8; 32],
        };

        let bytes = s.canonical_bytes().unwrap();
        let expected = br#"{"bond_pool_root":"0000000000000000000000000000000000000000000000000000000000000000","entropy_metric_scaled":"0","epoch_number":"0","impact_pool_root":"0000000000000000000000000000000000000000000000000000000000000000","kernel_hash":"0000000000000000000000000000000000000000000000000000000000000000","previous_root":"0000000000000000000000000000000000000000000000000000000000000000","validator_set_root":"0000000000000000000000000000000000000000000000000000000000000000","vdf_challenge_seed":"0000000000000000000000000000000000000000000000000000000000000000"}"#;
        assert_eq!(&bytes, expected,
            "canonical bytes diverged from expected — this is a serialization fork");
    }

    #[test]
    fn state_root_excluded_from_its_own_serialization() {
        // Changing state_root must NOT change the canonical bytes (no self-reference).
        let mut a = EpochState {
            bond_pool_root:        [0u8; 32],
            entropy_metric_scaled: 0,
            epoch_number:          0,
            impact_pool_root:      [0u8; 32],
            kernel_hash:           [0u8; 32],
            previous_root:         [0u8; 32],
            state_root:            [0u8; 32],
            validator_set_root:    [0u8; 32],
            vdf_challenge_seed:    [0u8; 32],
        };
        let b = EpochState { state_root: [0xFF; 32], ..a.clone() };
        assert_eq!(a.canonical_bytes().unwrap(), b.canonical_bytes().unwrap(),
            "state_root must not appear in its own canonical bytes");
    }

    #[test]
    fn field_change_changes_canonical_bytes() {
        // Mutating any non-state_root field must change the canonical bytes.
        let base = EpochState {
            bond_pool_root:        [0u8; 32],
            entropy_metric_scaled: 0,
            epoch_number:          0,
            impact_pool_root:      [0u8; 32],
            kernel_hash:           [0u8; 32],
            previous_root:         [0u8; 32],
            state_root:            [0u8; 32],
            validator_set_root:    [0u8; 32],
            vdf_challenge_seed:    [0u8; 32],
        };
        let modified = EpochState { epoch_number: 1, ..base.clone() };
        assert_ne!(base.canonical_bytes().unwrap(), modified.canonical_bytes().unwrap());

        let modified = EpochState { entropy_metric_scaled: 943_932_824_245, ..base.clone() };
        assert_ne!(base.canonical_bytes().unwrap(), modified.canonical_bytes().unwrap());
    }

    // ── Pinned constitutional hash vector ─────────────────────────────────────

    #[test]
    fn genesis_state_root_is_pinned() {
        // CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // This is the SHA-256 of the canonical JSON bytes of the all-zero genesis state.
        // Any change to the EpochState serialization format will break this test,
        // signalling a potential genesis fork.
        let s = EpochState {
            bond_pool_root:        [0u8; 32],
            entropy_metric_scaled: 0,
            epoch_number:          0,
            impact_pool_root:      [0u8; 32],
            kernel_hash:           [0u8; 32],
            previous_root:         [0u8; 32],
            state_root:            [0u8; 32],
            validator_set_root:    [0u8; 32],
            vdf_challenge_seed:    [0u8; 32],
        };
        let root = s.compute_state_root().unwrap();
        // PINNED CONSTITUTIONAL VECTOR — DO NOT CHANGE.
        // SHA-256(canonical JSON of all-zero genesis EpochState)
        // Changing ANY field name, order, or encoding rule breaks this assertion.
        let expected: [u8; 32] = [
            0xbb, 0x44, 0xf7, 0xd8, 0x3e, 0x9e, 0x4e, 0x42,
            0x68, 0x09, 0xa8, 0x1b, 0x66, 0xf7, 0x2a, 0x49,
            0x44, 0x32, 0x95, 0x4f, 0xbc, 0x05, 0xbf, 0x8f,
            0x07, 0x89, 0xa6, 0x23, 0xb1, 0xd5, 0xad, 0xe1,
        ];
        assert_eq!(root, expected, "genesis state_root diverged — serialization format changed");
        // Verify stability: compute twice, must be identical.
        assert_eq!(root, s.compute_state_root().unwrap(), "state_root must be deterministic");
    }

    // ── commit() API ──────────────────────────────────────────────────────────

    #[test]
    fn commit_assigns_correct_state_root() {
        let uncommitted = EpochState {
            bond_pool_root:        [0u8; 32],
            entropy_metric_scaled: 0,
            epoch_number:          1,
            impact_pool_root:      [0u8; 32],
            kernel_hash:           [0u8; 32],
            previous_root:         [0u8; 32],
            state_root:            [0u8; 32],  // placeholder, will be overwritten
            validator_set_root:    [0u8; 32],
            vdf_challenge_seed:    [0u8; 32],
        };
        let expected_root = uncommitted.compute_state_root().unwrap();
        let committed = uncommitted.commit().unwrap();
        assert_eq!(committed.state_root, expected_root);
        assert_ne!(committed.state_root, [0u8; 32],
            "committed state_root must not be all zeros");
    }

    // ── encode helpers ────────────────────────────────────────────────────────

    #[test]
    fn encode_u128_zero() {
        assert_eq!(encode_u128(0), b"0");
    }

    #[test]
    fn encode_u128_no_leading_zeros() {
        assert_eq!(encode_u128(1_000_000), b"1000000");
    }

    #[test]
    fn encode_u128_max() {
        assert_eq!(encode_u128(u128::MAX), b"340282366920938463463374607431768211455");
    }

    #[test]
    fn encode_digest_all_zeros() {
        let d = [0u8; 32];
        let s = encode_digest(&d);
        assert_eq!(s.len(), 64);
        assert!(s.iter().all(|&b| b == b'0'));
    }

    #[test]
    fn encode_digest_all_ff() {
        let d = [0xFFu8; 32];
        let s = encode_digest(&d);
        assert_eq!(s.len(), 64);
        assert!(s.iter().all(|&b| b == b'f'));
    }
}
