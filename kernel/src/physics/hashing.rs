//! SHA-256 binding for the Civilisation OS Kernel.
//!
//! CONSTITUTIONAL RULE: SHA-256 (FIPS 180-4). No other hash algorithm permitted.
//!
//! Implementation: self-contained reference implementation.
//! Zero external cryptographic dependencies.
//!
//! This implementation is intentionally unoptimized. Correctness and
//! cross-platform bit-exact determinism take absolute priority over performance.
//! It is a direct translation of the FIPS 180-4 specification, section 6.2.2.
//!
//! MERKLE DOMAIN SEPARATION:
//!   leaf_hash  = SHA256(0x00 || serialized_leaf_bytes)
//!   node_hash  = SHA256(0x01 || left_hash || right_hash)
//!
//! These prefix bytes prevent second-preimage attacks on the Merkle tree.
//! See: RFC 6962 §2.1 and the Merkle tree specification.

/// A SHA-256 digest: 32 bytes.
pub type Digest = [u8; 32];

/// The domain separation prefix for Merkle leaf hashes.
pub const LEAF_PREFIX: u8 = 0x00;

/// The domain separation prefix for Merkle internal node hashes.
pub const NODE_PREFIX: u8 = 0x01;

// ──────────────────────────────────────────────────────────────────────────────
// FIPS 180-4 §4.2.2 — SHA-256 initial hash values
// (First 32 bits of the fractional parts of the square roots of the first 8 primes)
// ──────────────────────────────────────────────────────────────────────────────
const H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

// ──────────────────────────────────────────────────────────────────────────────
// FIPS 180-4 §4.2.2 — SHA-256 round constants
// (First 32 bits of the fractional parts of the cube roots of the first 64 primes)
// ──────────────────────────────────────────────────────────────────────────────
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// FIPS 180-4 §4.1.2 — SHA-256 logical functions.
#[inline(always)]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ ((!x) & z)
}

#[inline(always)]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// FIPS 180-4 §4.1.2 — Σ₀ (big sigma 0)
#[inline(always)]
fn sigma0_upper(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// FIPS 180-4 §4.1.2 — Σ₁ (big sigma 1)
#[inline(always)]
fn sigma1_upper(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// FIPS 180-4 §4.1.2 — σ₀ (small sigma 0)
#[inline(always)]
fn sigma0_lower(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

/// FIPS 180-4 §4.1.2 — σ₁ (small sigma 1)
#[inline(always)]
fn sigma1_lower(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

/// Process one 512-bit (64-byte) message block.
/// Mutates state in place per FIPS 180-4 §6.2.2 steps 1–4.
fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    // Step 1: Prepare the message schedule W[0..63].
    let mut w = [0u32; 64];
    for t in 0..16 {
        w[t] = u32::from_be_bytes([
            block[t * 4],
            block[t * 4 + 1],
            block[t * 4 + 2],
            block[t * 4 + 3],
        ]);
    }
    for t in 16..64 {
        w[t] = sigma1_lower(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(sigma0_lower(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }

    // Step 2: Initialize the eight working variables.
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;

    // Step 3: 64 rounds.
    for t in 0..64 {
        let t1 = h
            .wrapping_add(sigma1_upper(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let t2 = sigma0_upper(a).wrapping_add(maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    // Step 4: Compute the new intermediate hash value.
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/// Feed a single byte into the pending buffer; compress if a full block is ready.
#[inline(always)]
fn feed_byte(state: &mut [u32; 8], pending: &mut [u8; 64], pending_len: &mut usize, byte: u8) {
    pending[*pending_len] = byte;
    *pending_len += 1;
    if *pending_len == 64 {
        compress(state, pending);
        *pending_len = 0;
    }
}

/// Compute SHA-256 over an arbitrary byte slice.
/// This is the canonical hash function for all Civilisation OS kernel operations.
/// Implements FIPS 180-4 §5.1.1 (padding) and §6.2.2 (hash computation).
pub fn sha256(input: &[u8]) -> Digest {
    let mut state = H;
    let bit_len: u64 = (input.len() as u64).wrapping_mul(8);

    let mut pending = [0u8; 64];
    let mut pending_len: usize = 0;

    // Feed all input bytes.
    for &byte in input {
        feed_byte(&mut state, &mut pending, &mut pending_len, byte);
    }

    // FIPS 180-4 §5.1.1 — append the single bit '1' (as 0x80 byte).
    feed_byte(&mut state, &mut pending, &mut pending_len, 0x80);

    // Pad with zero bytes until pending_len == 56 (so length fits in last 8 bytes).
    while pending_len != 56 {
        feed_byte(&mut state, &mut pending, &mut pending_len, 0x00);
    }

    // Append the original message length as a 64-bit big-endian integer.
    for byte in bit_len.to_be_bytes() {
        feed_byte(&mut state, &mut pending, &mut pending_len, byte);
    }

    // Produce the 256-bit (32-byte) digest.
    let mut digest = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// Hash a Merkle leaf: SHA256(0x00 || leaf_bytes)
pub fn hash_leaf(leaf_bytes: &[u8]) -> Digest {
    let mut input = Vec::with_capacity(1 + leaf_bytes.len());
    input.push(LEAF_PREFIX);
    input.extend_from_slice(leaf_bytes);
    sha256(&input)
}

/// Hash a Merkle internal node: SHA256(0x01 || left_hash || right_hash)
pub fn hash_node(left: &Digest, right: &Digest) -> Digest {
    let mut input = Vec::with_capacity(1 + 32 + 32);
    input.push(NODE_PREFIX);
    input.extend_from_slice(left);
    input.extend_from_slice(right);
    sha256(&input)
}

// ──────────────────────────────────────────────────────────────────────────────
// Constitutional test vectors from NIST FIPS 180-4 and NIST CAVP.
// These are byte-exact pinned values. Any deviation is a constitutional crisis.
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn nist(expected_hex: &str) -> Digest {
        assert_eq!(expected_hex.len(), 64, "SHA-256 hex must be exactly 64 chars");
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&expected_hex[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    #[test]
    fn raw_input_bytes_abc() {
        // b"abc" must be exactly ASCII 0x61, 0x62, 0x63. No encoding surprises.
        assert_eq!(b"abc", &[0x61u8, 0x62, 0x63]);
    }

    #[test]
    fn fips_vector_empty_string() {
        // NIST FIPS 180-4 / CAVP SHA-256("")
        assert_eq!(
            sha256(b""),
            nist("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }

    #[test]
    fn fips_vector_abc() {
        // NIST FIPS 180-4 / CAVP SHA-256("abc") — confirmed correct.
        // Verified against: NIST CAVP, RFC 4634, di-mgt.com.au SHA-256 reference.
        assert_eq!(
            sha256(b"abc"),
            nist("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn fips_vector_448bit_message() {
        // NIST FIPS 180-4 / CAVP SHA-256 of 448-bit message
        assert_eq!(
            sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            nist("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1")
        );
    }

    #[test]
    fn domain_separation_differs() {
        let leaf_h = hash_leaf(b"test");
        let node_h = hash_node(&sha256(b"test"), &sha256(b"test"));
        // The domain-separated hashes must differ even for related inputs.
        assert_ne!(leaf_h, node_h);
    }

    #[test]
    fn leaf_and_node_are_deterministic() {
        assert_eq!(hash_leaf(b"hello"), hash_leaf(b"hello"));
        let d = sha256(b"x");
        assert_eq!(hash_node(&d, &d), hash_node(&d, &d));
    }
}
