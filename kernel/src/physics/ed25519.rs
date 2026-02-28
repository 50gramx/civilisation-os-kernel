//! Ed25519 signature verification — thin wrapper around ed25519-dalek.
//!
//! CONSTITUTIONAL RULE: This is the ONLY Ed25519 implementation permitted.
//!
//! WHY VENDORED (not custom):
//!   Custom implementation produced four consensus-fork-class bugs
//!   (u128 overflow, reduction carry chain, exponent chain, Barrett truncation)
//!   before test vectors could even finish executing.
//!   Crypto must be predictable infrastructure, not original code.
//!
//! AUDIT SURFACE:
//!   ed25519-dalek 2.1.1 — MIT/Apache-2.0 — most widely used Ed25519 in Rust.
//!   curve25519-dalek uses proven 5×51-bit radix field arithmetic.
//!   All six hardening invariants enforced by dalek:
//!     1. Small-order public key rejection
//!     2. Canonical S scalar (S < L)
//!     3. Canonical R encoding
//!     4. Non-canonical field element rejection
//!     5. Constant-time comparison
//!     6. Cofactored verification (via verify_strict)
//!
//! PINNED: ed25519-dalek = "=2.1.1", default-features = false

use crate::TransitionError;

/// Verify an Ed25519 signature.
///
/// - `pubkey`: 32-byte compressed Edwards point
/// - `message`: arbitrary-length message
/// - `signature`: 64-byte signature (R || s)
///
/// Uses `verify_strict` which enforces all six hardening invariants:
/// cofactored verification, canonical encoding checks, and small-order rejection.
///
/// Returns `Ok(())` if valid, `Err(InvalidSignature)` otherwise.
pub fn verify(
    pubkey: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), TransitionError> {
    use ed25519_dalek::{Signature, VerifyingKey};

    let vk = VerifyingKey::from_bytes(pubkey)
        .map_err(|_| TransitionError::InvalidSignature)?;

    let sig = Signature::from_bytes(signature);

    vk.verify_strict(message, &sig)
        .map_err(|_| TransitionError::InvalidSignature)
}

// ──────────────────────────────────────────────────────────────────────────────
// RFC 8032 §6 pinned test vectors — CONSTITUTIONAL, DO NOT CHANGE
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn from_hex_32(s: &str) -> [u8; 32] {
        assert_eq!(s.len(), 64);
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    fn from_hex_64(s: &str) -> [u8; 64] {
        assert_eq!(s.len(), 128);
        let mut out = [0u8; 64];
        for i in 0..64 {
            out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    // RFC 8032 §6.1 Test Vector 1 — empty message
    #[test]
    fn rfc8032_vector_1_empty_message() {
        let pubkey = from_hex_32(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        );
        let signature = from_hex_64(
            "e5564300c360ac729086e2cc806e828a\
             84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46\
             bd25bf5f0595bbe24655141438e7a100b",
        );
        assert_eq!(
            verify(&pubkey, b"", &signature),
            Ok(()),
            "RFC 8032 vector 1 (empty message) must verify"
        );
    }

    // RFC 8032 §6.1 Test Vector 2 — 1-byte message 0x72
    #[test]
    fn rfc8032_vector_2_one_byte_message() {
        let pubkey = from_hex_32(
            "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
        );
        let signature = from_hex_64(
            "92a009a9f0d4cab8720e820b5f642540\
             a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8\
             c387b2eaeb4302aeeb00d291612bb0c00",
        );
        assert_eq!(
            verify(&pubkey, &[0x72], &signature),
            Ok(()),
            "RFC 8032 vector 2 (1-byte message 0x72) must verify"
        );
    }

    // Mutated signature must fail
    #[test]
    fn mutated_signature_fails() {
        let pubkey = from_hex_32(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        );
        let mut signature = from_hex_64(
            "e5564300c360ac729086e2cc806e828a\
             84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46\
             bd25bf5f0595bbe24655141438e7a100b",
        );
        signature[0] ^= 0x01;
        assert_eq!(
            verify(&pubkey, b"", &signature),
            Err(TransitionError::InvalidSignature),
            "mutated signature must fail"
        );
    }

    // Wrong message must fail
    #[test]
    fn wrong_message_fails() {
        let pubkey = from_hex_32(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        );
        let signature = from_hex_64(
            "e5564300c360ac729086e2cc806e828a\
             84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46\
             bd25bf5f0595bbe24655141438e7a100b",
        );
        // Signed over empty message — verify against non-empty must fail.
        assert_eq!(
            verify(&pubkey, b"wrong", &signature),
            Err(TransitionError::InvalidSignature),
            "wrong message must fail"
        );
    }

    // Invalid pubkey (all zeros) must fail
    #[test]
    fn invalid_pubkey_fails() {
        let pubkey = [0u8; 32];
        let signature = [0u8; 64];
        assert_eq!(
            verify(&pubkey, b"test", &signature),
            Err(TransitionError::InvalidSignature),
            "invalid pubkey must fail"
        );
    }
}
