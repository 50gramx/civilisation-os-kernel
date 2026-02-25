//! Absolute rewind logic: reverts state to previous_root on proven fraud.
//!
//! CONSTITUTIONAL RULES:
//! - MAX_FRAUD_WINDOW_EPOCHS = 1. Only the immediately preceding epoch is rewindable.
//! - VDF seed is recomputed from X-1 state exclusively. Fraudulent seed is severed.
//! - FraudProofs are processed in ascending lexicographical order of their JCS hash.
//! - Rewind across kernel_hash boundaries is forbidden.

// Stub for v0.0.0-draft. Full implementation in v0.0.1-alpha.
