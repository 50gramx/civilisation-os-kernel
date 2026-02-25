//! Perfect Binary Padded Merkle Tree.
//!
//! CONSTITUTIONAL RULES (all frozen):
//! 1. Leaf Ordering:     Leaves are sorted lexicographically by their byte content BEFORE hashing.
//! 2. Empty Tree:        Zero leaves → root = SHA256(0x00 || []) (leaf-prefix hash of empty bytes).
//! 3. Depth Padding:     If leaf_count > 0, pad to next power of two by duplicating the final node.
//! 4. MAX_MERKLE_DEPTH:  40 levels. Input exceeding 2^40 leaves is a TransitionError.
//! 5. Domain Separation: leaf_hash = SHA256(0x00 || leaf), node_hash = SHA256(0x01 || L || R).

use std::vec::Vec;
use crate::TransitionError;
use crate::physics::hashing::{Digest, hash_leaf, hash_node, sha256, LEAF_PREFIX};

/// Maximum allowed Merkle tree depth. Supports up to 2^40 ≈ 1_099_511_627_776 leaves.
pub const MAX_MERKLE_DEPTH: usize = 40;

/// The root of an empty tree (zero leaves).
/// = SHA256(0x00 || [])
pub fn empty_tree_root() -> Digest {
    sha256(&[LEAF_PREFIX])
}

/// Compute the Merkle root over a collection of already-serialized leaf byte slices.
///
/// The caller is responsible for pre-sorting leaves lexicographically before calling.
/// This function does NOT sort — sorting is the caller's constitutional obligation.
///
/// Returns TransitionError::PayloadLimitExceeded if leaves.len() > 2^MAX_MERKLE_DEPTH.
pub fn compute_merkle_root(leaves: &[Vec<u8>]) -> Result<Digest, TransitionError> {
    if leaves.is_empty() {
        return Ok(empty_tree_root());
    }

    // Guard: check we don't exceed the maximum tree depth.
    // 2^40 = 1_099_511_627_776. If leaves exceed this, reject immediately.
    let max_leaves = 1u128 << MAX_MERKLE_DEPTH;
    if leaves.len() as u128 > max_leaves {
        return Err(TransitionError::PayloadLimitExceeded);
    }

    // Hash all leaves with domain separation.
    let mut nodes: Vec<Digest> = leaves.iter().map(|l| hash_leaf(l)).collect();

    // Pad to next power of two by duplicating the final node.
    let padded_len = next_power_of_two(nodes.len());
    while nodes.len() < padded_len {
        let last = *nodes.last().unwrap(); // safe: nodes is non-empty here
        nodes.push(last);
    }

    // Build the tree bottom-up until one root remains.
    while nodes.len() > 1 {
        let mut next_level: Vec<Digest> = Vec::with_capacity(nodes.len() / 2);
        for pair in nodes.chunks_exact(2) {
            next_level.push(hash_node(&pair[0], &pair[1]));
        }
        // If after reduction we still have an odd count (shouldn't happen after padding,
        // but defensive duplication per spec: duplicate the final node).
        if next_level.len() % 2 != 0 && next_level.len() > 1 {
            let last = *next_level.last().unwrap();
            next_level.push(last);
        }
        nodes = next_level;
    }

    Ok(nodes[0])
}

/// Returns the smallest power of two >= n. Returns 1 for n == 0.
fn next_power_of_two(n: usize) -> usize {
    if n <= 1 { return 1; }
    let mut result = 1usize;
    while result < n {
        result <<= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_is_deterministic() {
        assert_eq!(empty_tree_root(), empty_tree_root());
    }

    #[test]
    fn single_leaf_root_equals_leaf_hash() {
        let leaf = b"hello";
        let root = compute_merkle_root(&[leaf.to_vec()]).unwrap();
        // For a single leaf, root = hash_node(hash_leaf(leaf), hash_leaf(leaf))
        // because we pad to power of 2 (1 -> 1, no change, then 1-node tree = the hash itself? no wait)
        // Actually: 1 leaf → padded_len = 1 → nodes = [hash_leaf(leaf)] → already len 1 → return that.
        let expected = hash_leaf(leaf);
        // Wait: padded_len of 1 = 1, so no padding. nodes = [hash_leaf(leaf)].
        // While loop runs: nodes.len() = 1 → exit. root = nodes[0] = hash_leaf(leaf).
        assert_eq!(root, expected);
    }

    #[test]
    fn two_leaf_tree() {
        let a = b"aaa".to_vec();
        let b = b"bbb".to_vec();
        let root = compute_merkle_root(&[a.clone(), b.clone()]).unwrap();
        let expected = hash_node(&hash_leaf(&a), &hash_leaf(&b));
        assert_eq!(root, expected);
    }

    #[test]
    fn three_leaves_pads_to_four() {
        // 3 leaves → pad to 4 by duplicating the 3rd
        let leaves: Vec<Vec<u8>> = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let root = compute_merkle_root(&leaves).unwrap();
        // Manually compute:
        let h0 = hash_leaf(b"a");
        let h1 = hash_leaf(b"b");
        let h2 = hash_leaf(b"c");
        let h3 = h2; // duplicate
        let n01 = hash_node(&h0, &h1);
        let n23 = hash_node(&h2, &h3);
        let expected = hash_node(&n01, &n23);
        assert_eq!(root, expected);
    }

    #[test]
    fn ordering_matters() {
        let leaves_ab = vec![b"a".to_vec(), b"b".to_vec()];
        let leaves_ba = vec![b"b".to_vec(), b"a".to_vec()];
        let root_ab = compute_merkle_root(&leaves_ab).unwrap();
        let root_ba = compute_merkle_root(&leaves_ba).unwrap();
        // Pre-sorting is caller's responsibility. Different order → different root.
        assert_ne!(root_ab, root_ba);
    }
}
