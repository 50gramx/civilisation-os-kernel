//! SHA-512 — FIPS 180-4 §6.4 reference implementation.
//!
//! CONSTITUTIONAL RULE: This is the only SHA-512 permitted in the kernel.
//! Required by Ed25519 (H = SHA-512(R || pk || msg) in RFC 8032 §5.1).
//!
//! Implementation mirrors `physics/hashing.rs` (SHA-256) in structure.
//! Correctness and cross-platform bit-exact determinism take absolute
//! priority over performance. Direct translation of FIPS 180-4.
//!
//! FIPS 180-4 test vectors pinned below.

/// A SHA-512 digest: 64 bytes.
pub type Digest512 = [u8; 64];

// ──────────────────────────────────────────────────────────────────────────────
// FIPS 180-4 §5.3.5 — SHA-512 initial hash values
// (First 64 bits of the fractional parts of the square roots of the first 8 primes)
// ──────────────────────────────────────────────────────────────────────────────
const H: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

// ──────────────────────────────────────────────────────────────────────────────
// FIPS 180-4 §4.2.3 — SHA-512 round constants (80 words)
// (First 64 bits of the fractional parts of the cube roots of the first 80 primes)
// ──────────────────────────────────────────────────────────────────────────────
const K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

// ──────────────────────────────────────────────────────────────────────────────
// FIPS 180-4 §4.1.3 — SHA-512 logical functions
// ──────────────────────────────────────────────────────────────────────────────

#[inline(always)]
fn ch(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ ((!x) & z)
}

#[inline(always)]
fn maj(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// FIPS 180-4 §4.1.3 — Σ₀⁵¹² (big sigma 0)
#[inline(always)]
fn sigma0_upper(x: u64) -> u64 {
    x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
}

/// FIPS 180-4 §4.1.3 — Σ₁⁵¹² (big sigma 1)
#[inline(always)]
fn sigma1_upper(x: u64) -> u64 {
    x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
}

/// FIPS 180-4 §4.1.3 — σ₀⁵¹² (small sigma 0)
#[inline(always)]
fn sigma0_lower(x: u64) -> u64 {
    x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
}

/// FIPS 180-4 §4.1.3 — σ₁⁵¹² (small sigma 1)
#[inline(always)]
fn sigma1_lower(x: u64) -> u64 {
    x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
}

/// Process one 1024-bit (128-byte) message block.
fn compress(state: &mut [u64; 8], block: &[u8; 128]) {
    // Step 1: Prepare the message schedule W[0..79].
    let mut w = [0u64; 80];
    for t in 0..16 {
        w[t] = u64::from_be_bytes([
            block[t * 8],     block[t * 8 + 1], block[t * 8 + 2], block[t * 8 + 3],
            block[t * 8 + 4], block[t * 8 + 5], block[t * 8 + 6], block[t * 8 + 7],
        ]);
    }
    for t in 16..80 {
        w[t] = sigma1_lower(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(sigma0_lower(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }

    // Step 2: Initialize eight working variables.
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;

    // Step 3: 80 rounds.
    for t in 0..80 {
        let t1 = h
            .wrapping_add(sigma1_upper(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let t2 = sigma0_upper(a).wrapping_add(maj(a, b, c));
        h = g; g = f; f = e;
        e = d.wrapping_add(t1);
        d = c; c = b; b = a;
        a = t1.wrapping_add(t2);
    }

    // Step 4: Compute new intermediate hash.
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

#[inline(always)]
fn feed_byte(state: &mut [u64; 8], pending: &mut [u8; 128], pending_len: &mut usize, byte: u8) {
    pending[*pending_len] = byte;
    *pending_len += 1;
    if *pending_len == 128 {
        compress(state, pending);
        *pending_len = 0;
    }
}

/// Compute SHA-512 over an arbitrary byte slice.
/// FIPS 180-4 §5.1.2 (padding) and §6.4 (hash computation).
pub fn sha512(input: &[u8]) -> Digest512 {
    let mut state = H;
    // Message length in bits — SHA-512 uses a 128-bit length field, but
    // inputs here never exceed u64::MAX bytes, so hi=0 is always safe.
    let bit_len_lo: u64 = (input.len() as u64).wrapping_mul(8);
    let bit_len_hi: u64 = 0;

    let mut pending = [0u8; 128];
    let mut pending_len: usize = 0;

    for &b in input {
        feed_byte(&mut state, &mut pending, &mut pending_len, b);
    }

    // FIPS 180-4 §5.1.2 — append the single bit '1' (as 0x80).
    feed_byte(&mut state, &mut pending, &mut pending_len, 0x80);

    // Pad with zeros until pending_len == 112 (so 16-byte length fits last 16 bytes).
    while pending_len != 112 {
        feed_byte(&mut state, &mut pending, &mut pending_len, 0x00);
    }

    // Append the original message length as a 128-bit big-endian integer.
    for byte in bit_len_hi.to_be_bytes() {
        feed_byte(&mut state, &mut pending, &mut pending_len, byte);
    }
    for byte in bit_len_lo.to_be_bytes() {
        feed_byte(&mut state, &mut pending, &mut pending_len, byte);
    }

    // Produce the 512-bit (64-byte) digest.
    let mut digest = [0u8; 64];
    for (i, word) in state.iter().enumerate() {
        digest[i * 8..i * 8 + 8].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

// ──────────────────────────────────────────────────────────────────────────────
// FIPS 180-4 test vectors — DO NOT CHANGE
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn from_hex(s: &str) -> [u8; 64] {
        assert_eq!(s.len(), 128);
        let mut out = [0u8; 64];
        for i in 0..64 {
            out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    #[test]
    fn fips_vector_empty_string() {
        // NIST FIPS 180-4 SHA-512("") — CONSTITUTIONAL VECTOR.
        assert_eq!(
            sha512(b""),
            from_hex(
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce\
                 47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
            )
        );
    }

    #[test]
    fn fips_vector_abc() {
        // NIST FIPS 180-4 SHA-512("abc") — CONSTITUTIONAL VECTOR.
        assert_eq!(
            sha512(b"abc"),
            from_hex(
                "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
                 2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
            )
        );
    }

    #[test]
    fn fips_vector_448bit_message() {
        // NIST FIPS 180-4 SHA-512 of 448-bit message (same as abc+longer).
        assert_eq!(
            sha512(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            from_hex(
                "204a8fc6dda82f0a0ced7beb8e08a41657c16ef468b228a8279be331a703c335\
                 96fd15c13b1b07f9aa1d3bea57789ca031ad85c7a71dd70354ec631238ca3445"
            )
        );
    }

    #[test]
    fn sha512_is_deterministic() {
        assert_eq!(sha512(b"hello"), sha512(b"hello"));
    }

    #[test]
    fn sha512_differs_from_sha256() {
        use crate::physics::hashing::sha256;
        let input = b"test";
        let h256 = sha256(input);
        let h512 = sha512(input);
        // SHA-256 is 32 bytes, SHA-512 is 64 bytes — first 32 bytes must differ.
        assert_ne!(h256[..], h512[..32]);
    }
}
