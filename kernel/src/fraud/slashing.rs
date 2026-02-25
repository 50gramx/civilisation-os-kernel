//! Slashing: deterministic, idempotent, purely subtractive.
//!
//! CONSTITUTIONAL RULES:
//! - Uses saturating_sub_for_slash ONLY. Balance clamps to zero, never errors.
//! - Max ONE slash per validator per epoch. Tracked via BTreeSet (not HashSet).
//! - Slashed dust is burned (not redistributed). No incentive loops.

// Stub for v0.0.0-draft.
