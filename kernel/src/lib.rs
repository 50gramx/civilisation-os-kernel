//! Civilisation OS — Deterministic State Transition Kernel
//! Version: 0.0.0-draft
//!
//! Physics engine. No networking, no I/O, no async, no threading.
//! (v0.0.0-draft uses std. The no_std constraint is enforced when building
//! for wasm32-unknown-unknown via `cargo build --target wasm32-unknown-unknown`.)
//!
//! INVARIANTS:
//! 1. All arithmetic uses checked_* variants. Overflow → TransitionError::MathOverflow.
//! 2. All Fixed(u128) inner values are private. No raw .0 access in consensus paths.
//! 3. BTreeMap used everywhere: iteration order is deterministic (sorted by key).
//! 4. HashMap is forbidden in consensus code (random seed = non-determinism).
//! 5. Feature flags that alter execution semantics are constitutionally forbidden.
//! 6. Floating-point arithmetic is forbidden. All math goes through the Fixed type.

pub mod compat;
pub mod math;
pub mod physics;
pub mod state;
pub mod transition;
pub mod fraud;
pub mod emission;

/// The canonical error type for all state transition failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    /// Checked arithmetic returned None: overflow or underflow.
    MathOverflow,
    /// Division by zero pre-check triggered.
    DivisionByZero,
    /// JSON payload violated canonical serialization rules (RFC 8785 / JCS).
    InvalidSerialization,
    /// JSON object contained a duplicate key — constitutionally forbidden.
    DuplicateKey,
    /// Merkle witness verification failed or depth exceeded MAX_MERKLE_DEPTH (40).
    InvalidMerkleWitness,
    /// VDF SNARK proof failed verification.
    InvalidVdfProof,
    /// Ed25519 signature invalid.
    InvalidSignature,
    /// VouchBond magnitude below MIN_BOND_MAGNITUDE (1 * SCALE).
    BondTooSmall,
    /// Epoch payload count exceeded MAX_PAYLOADS_PER_EPOCH (10,000).
    PayloadLimitExceeded,
    /// FraudProof references an epoch outside MAX_FRAUD_WINDOW_EPOCHS (1).
    FraudWindowExpired,
    /// Snapshot kernel hash diverges from current kernel.
    KernelHashMismatch,
}
