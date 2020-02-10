extern crate alloc;

pub mod merge;
pub mod proof;
pub mod proof_map;

mod common;

use alloc::boxed::Box;
use common::split_slice;
use merge::Merge;
use proof::ProofElem;

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

/// Check whether proof proves leaf to be in root.
pub fn verify_proof<M: Merge>(leaf: &M::Hash, root: &M::Hash, proof: &[ProofElem<M::Hash>]) -> bool
where
    M::Hash: Eq,
{
    proof
        .iter()
        .fold(M::leaf(leaf), |subroot, proof_element| {
            proof_element.merge::<M>(&subroot)
        })
        .eq(root)
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
