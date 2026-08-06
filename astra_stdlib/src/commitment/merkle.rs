//! Binary Merkle tree over the BLS12-381 scalar field, using the local
//! [`crate::hash::poseidon::poseidon2`] as the node hash.
//!
//! Versioned `v1`. `commit` pads the leaf set to the next power of two with
//! zero leaves, so a tree's root is well-defined for any leaf count.

use bls12_381::Scalar;
use ff::Field;

use crate::hash::poseidon::poseidon2;

/// A Merkle inclusion proof for one leaf: the sibling path from the leaf up to
/// the root, plus a bit per level indicating whether the sibling is on the
/// left (`true`) or right (`false`) of the traversed node.
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub path: Vec<Scalar>,
    pub left_sibling: Vec<bool>,
}

/// Hash two children into a parent.
pub fn node_hash(left: Scalar, right: Scalar) -> Scalar {
    poseidon2(left, right)
}

/// Build the padded tree and return its root.
pub fn commit(leaves: &[Scalar]) -> Scalar {
    let mut level: Vec<Scalar> = leaves.to_vec();
    let padded = level.len().next_power_of_two();
    level.resize(padded, Scalar::ZERO);
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks_exact(2) {
            next.push(node_hash(pair[0], pair[1]));
        }
        level = next;
    }
    level[0]
}

/// Build the inclusion proof for `leaf_index` in the same padded tree as
/// [`commit`].
pub fn proof(leaves: &[Scalar], leaf_index: usize) -> MerkleProof {
    let mut level: Vec<Scalar> = leaves.to_vec();
    let padded = level.len().next_power_of_two();
    level.resize(padded, Scalar::ZERO);

    let mut path = Vec::new();
    let mut left_sibling = Vec::new();
    let mut idx = leaf_index;

    while level.len() > 1 {
        let sibling = if idx.is_multiple_of(2) {
            level[idx + 1]
        } else {
            level[idx - 1]
        };
        path.push(sibling);
        left_sibling.push(idx % 2 == 1);

        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks_exact(2) {
            next.push(node_hash(pair[0], pair[1]));
        }
        level = next;
        idx /= 2;
    }

    MerkleProof { path, left_sibling }
}

/// Recompute the root from a leaf + proof and compare to the expected root.
pub fn open(root: Scalar, leaf: Scalar, proof: &MerkleProof) -> bool {
    let mut cur = leaf;
    for (i, sibling) in proof.path.iter().enumerate() {
        let on_right = proof.left_sibling[i];
        cur = if on_right {
            node_hash(*sibling, cur)
        } else {
            node_hash(cur, *sibling)
        };
    }
    cur == root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_single_leaf() {
        // A single leaf is already a power of two, so the root is the leaf.
        assert_eq!(commit(&[Scalar::from(9u64)]), Scalar::from(9u64));
        // Two leaves are hashed into a parent.
        assert_eq!(
            commit(&[Scalar::from(1u64), Scalar::from(2u64)]),
            node_hash(Scalar::from(1u64), Scalar::from(2u64))
        );
    }

    #[test]
    fn inclusion_proof_verifies() {
        let leaves: Vec<Scalar> = (1..=8).map(Scalar::from).collect();
        let root = commit(&leaves);
        for idx in 0..leaves.len() {
            let p = proof(&leaves, idx);
            assert!(
                open(root, leaves[idx], &p),
                "leaf {idx} should verify against the root"
            );
        }
    }

    #[test]
    fn wrong_leaf_rejected() {
        let leaves: Vec<Scalar> = (1..=8).map(Scalar::from).collect();
        let root = commit(&leaves);
        let p = proof(&leaves, 0);
        assert!(!open(root, Scalar::from(42u64), &p));
    }

    #[test]
    fn tampered_path_rejected() {
        let leaves: Vec<Scalar> = (1..=4).map(Scalar::from).collect();
        let root = commit(&leaves);
        let mut p = proof(&leaves, 0);
        p.path[0] += Scalar::ONE;
        assert!(!open(root, leaves[0], &p));
    }
}
