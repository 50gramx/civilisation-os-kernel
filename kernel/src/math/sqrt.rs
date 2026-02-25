//! Constitutional integer square root — Babylonian method, floor-rounded.
//!
//! This implementation is pinned verbatim. It must NOT be replaced with a
//! crate dependency, a platform intrinsic, or a floating-point approximation.
//! The algorithm uses only u128 integer division (truncation = floor),
//! which is identical across all Rust targets including wasm32-unknown-unknown.
//!
//! DETERMINISM GUARANTEE:
//! Given identical input n, isqrt(n) produces identical output on:
//!   - Linux x86_64
//!   - macOS aarch64
//!   - Windows x86_64
//!   - wasm32-unknown-unknown
//!
//! The test vectors below are CONSTITUTIONAL. Any implementation that does not
//! produce these exact outputs for these exact inputs is non-compliant.

/// Returns floor(sqrt(n)) for any u128 input.
/// Uses the Babylonian (Newton's method) integer convergence.
///
/// Edge cases:
///   isqrt(0) = 0
///   isqrt(1) = 1
///   isqrt(u128::MAX) = 18_446_744_073_709_551_615
pub fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    // Initial estimate: use bit-length to start near the answer.
    // This avoids the slow convergence that occurs when starting from n itself.
    let bits = 128 - n.leading_zeros();
    let mut x = 1u128 << ((bits + 1) / 2);
    loop {
        // Babylonian step: next = (x + n/x) / 2
        // Integer division here truncates — this is intentional and constitutional.
        let next = (x + n / x) / 2;
        if next >= x {
            // Converged: x is now floor(sqrt(n)).
            return x;
        }
        x = next;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Constitutional test vectors.
// These values are FROZEN. They define what "correct sqrt" means for this kernel.
// Any platform that produces different results for these inputs is non-compliant.
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constitutional_vectors() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(2), 1);   // floor(1.41..) = 1
        assert_eq!(isqrt(3), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(10), 3);  // floor(3.16..) = 3
        assert_eq!(isqrt(100), 10);
        // SCALE = 10^12:  isqrt(SCALE) = 1_000_000
        assert_eq!(isqrt(1_000_000_000_000u128), 1_000_000);
        // isqrt(SCALE^2) = SCALE exactly
        assert_eq!(isqrt(1_000_000_000_000_000_000_000_000u128), 1_000_000_000_000);
        // u128::MAX = 2^128 - 1. floor(sqrt(u128::MAX)) = 2^64 - 1.
        assert_eq!(isqrt(u128::MAX), 18_446_744_073_709_551_615u128);
    }

    #[test]
    fn floor_property() {
        // For any perfect square n^2, isqrt(n^2) == n.
        // For n^2 + 1 (where n > 0), isqrt must still return n (floor).
        // Note: n=0 is excluded from the sq+1 check because isqrt(1) = 1, not 0.
        for n in [1u128, 100, 99991, 1_000_000, 1_000_000_000] {
            let sq = n * n;
            assert_eq!(isqrt(sq), n, "isqrt({}^2) should be {}", n, n);
            if sq < u128::MAX {
                assert_eq!(isqrt(sq + 1), n, "isqrt({}^2 + 1) should still be {}", n, n);
            }
        }
        // n=0 edge: isqrt(0) = 0, isqrt(1) = 1 (not 0)
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
    }
}
