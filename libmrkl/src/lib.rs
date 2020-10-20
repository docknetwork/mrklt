//! ```
//! # // This is a dummy hash function. Don't use this!
//! # fn hash(c: &[&[u8]]) -> [u8; 32] {
//! #     let a = c.iter().map(|a| *a).flatten().cloned().fold(0, u8::wrapping_add);
//! #     [a; 32]
//! # }
//! use libmrkl::{compute_root, create_proof, Merge, verify_proof};
//!
//! struct MyHash;
//!
//! impl Merge for MyHash {
//!     type Hash = [u8; 32];
//!
//!     fn leaf(leaf: &[u8; 32]) -> [u8; 32] {
//!         hash(&[leaf])
//!     }
//!
//!     fn merge(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
//!         hash(&[left, right])
//!     }
//! }
//!
//! let leaves = [[1u8; 32], [2u8; 32], [1u8; 32], [3u8; 32]];
//! let root = compute_root::<MyHash>(&leaves);
//! let proof = create_proof::<MyHash>(1, &leaves);
//! assert_eq!(root, verify_proof::<MyHash>(&leaves[1], &proof));
//! ```

#![no_std]

extern crate alloc;

mod common;
mod merge;
pub mod proof;
pub mod proof_map;

use alloc::boxed::Box;
use common::split_slice;
pub use merge::Merge;
use proof::ProofElem;

/// Deterministically compute a Merkle root for an ordered list of leaves.
///
/// # Panics
///
/// Panics if length of leaves is 0.
pub fn compute_root<M: Merge>(leaves: &[M::Hash]) -> M::Hash {
    match leaves.len() {
        0 => panic!(),
        1 => M::leaf(&leaves[0]),
        _ => {
            let (left, right) = split_slice(leaves);
            M::merge(&compute_root::<M>(left), &compute_root::<M>(right))
        }
    }
}

/// Create the proof of inclusion for the indexed leaf.
///
/// # Panics
///
/// Panics if leaf_index is >= leaves.len().
pub fn create_proof<M: Merge>(leaf_index: usize, leaves: &[M::Hash]) -> Box<[ProofElem<M::Hash>]>
where
    M::Hash: Clone,
{
    let a = proof_map::HashCache::from_leaves::<M>(leaves);
    a.create_proof(leaf_index)
}

/// Calculate the expected Merkle root given the a leaf and its proof.
pub fn verify_proof<M: Merge>(leaf: &M::Hash, proof: &[ProofElem<M::Hash>]) -> M::Hash {
    proof.iter().fold(M::leaf(leaf), |subroot, proof_element| {
        proof_element.merge::<M>(&subroot)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sum_as_expected() {
        struct Sum;
        impl Merge for Sum {
            type Hash = u8;
            fn leaf(a: &u8) -> u8 {
                *a
            }
            fn merge(a: &u8, b: &u8) -> u8 {
                a.wrapping_add(*b)
            }
        }

        struct DoubleThenSum;
        impl Merge for DoubleThenSum {
            type Hash = u8;
            fn leaf(a: &u8) -> u8 {
                a + a
            }
            fn merge(a: &u8, b: &u8) -> u8 {
                a.wrapping_add(*b)
            }
        }

        let tocheck: &[&[u8]] = &[
            &[0u8][..],
            &[0u8, 1][..],
            &[0u8, 1, 2][..],
            &[0u8, 1, 2, 3][..],
            &[0u8, 1, 2, 3, 4][..],
            &[0u8, 1, 2, 3, 4, 5][..],
        ];
        for a in tocheck {
            assert_eq!(compute_root::<Sum>(a), a.iter().sum());
            assert_eq!(compute_root::<DoubleThenSum>(a), a.iter().sum::<u8>() * 2);
        }
    }
}
