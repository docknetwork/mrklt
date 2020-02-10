use super::common::{split_range, split_tuple_as_range};
use super::merge::Merge;
use super::proof::ProofElem;
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::ops::Range;
use core::prelude::v1::*;

pub struct HashCache<T> {
    cache: BTreeMap<(usize, usize), T>,
    leaves_len: usize,
}

impl<T> HashCache<T> {
    /// # Panics
    ///
    /// Panics if length of leaves is 0.
    pub fn from_leaves<M: Merge<Hash = T>>(leaves: &[T]) -> Self {
        let mut cache = Default::default();
        let root = populate::<M>(&mut cache, leaves, 0, leaves.len());
        cache.insert((0, leaves.len()), root);
        HashCache {
            cache,
            leaves_len: leaves.len(),
        }
    }

    /// Get the precalculated merkle root.
    pub fn root(&self) -> &T {
        &self.cache[&(0, self.leaves_len)]
    }

    /// Get a proof of inclusion for the leaf at index.
    ///
    /// # Panics
    ///
    /// Panics if index >= the length of the original list of leaves.
    pub fn create_proof(&self, index: usize) -> Box<[ProofElem<T>]>
    where
        T: Clone,
    {
        // depth = sqrt(len+1)-1 but usize.sqrt() is not defined so preallocating correctly is not
        // trivial.
        let mut ret: Vec<ProofElem<T>> = Vec::new();
        self.populate_proof(&mut ret, index, 0..self.leaves_len);
        ret.into_boxed_slice()
    }

    /// # Panics
    ///
    /// Panics if !range.contains(index) or if range is not in cache.
    fn populate_proof(&self, proof: &mut Vec<ProofElem<T>>, index: usize, range: Range<usize>)
    where
        T: Clone,
    {
        // assert range.len() != 0 and that index is in range
        if !range.contains(&index) {
            panic!("index not in range");
        }
        if range.len() == 1 {
            return;
        }
        let (left, right) = split_range(range);
        let in_left = left.contains(&index);
        debug_assert!(in_left ^ right.contains(&index));
        if in_left {
            self.populate_proof(proof, index, left);
            let h = self.cache[&(right.start, right.end)].clone();
            proof.push(ProofElem::Right(h));
        } else {
            self.populate_proof(proof, index, right);
            let h = self.cache[&(left.start, left.end)].clone();
            proof.push(ProofElem::Left(h));
        }
    }
}

/// Populate hashcache. Does not add the root hash. If the caller needs the root hash to be
/// included. They should do this:
///
/// ```nocompile
/// let root = populate(cache, leaves, 0, leaves.len());
/// cache.insert((0, leaves.len()), root);
/// ```
///
/// Panics if end >= start.
fn populate<M: Merge>(
    cache: &mut BTreeMap<(usize, usize), M::Hash>,
    leaves: &[M::Hash],
    start: usize,
    end: usize,
) -> M::Hash {
    match end.checked_sub(start) {
        None | Some(0) => panic!(),
        Some(1) => M::leaf(&leaves[start]),
        Some(_) => {
            let (left, right) = split_tuple_as_range(start, end);
            let lefth = populate::<M>(cache, leaves, left.0, left.1);
            let righth = populate::<M>(cache, leaves, right.0, right.1);
            cache.insert(left, lefth);
            cache.insert(right, righth);
            M::merge(&cache[&left], &cache[&right])
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::compute_root;
    use super::super::verify_proof;
    use super::*;
    use blake2::{Blake2s, Digest};
    use rand::distributions::Distribution;
    use rand::distributions::Standard;

    fn intersection(a: &Range<usize>, b: &Range<usize>) -> Range<usize> {
        let start = a.start.max(b.start);
        let end = a.end.min(b.end);
        let end = end.max(start);
        start..end
    }

    fn ra(tup: &(usize, usize)) -> Range<usize> {
        tup.0..tup.1
    }

    fn happy_path<M: Merge, Ls>(leaf_sets: Ls)
    where
        M::Hash: Clone + Eq,
        Ls: Iterator<Item = Vec<M::Hash>>,
    {
        for leaves in leaf_sets {
            let hc = HashCache::from_leaves::<M>(&leaves);
            let root = hc.root();
            assert!(root == &compute_root::<M>(&leaves));
            for (index, leaf) in leaves.iter().enumerate() {
                let proof = hc.create_proof(index);
                assert!(verify_proof::<M>(leaf, root, &proof));
            }

            // expected properties of hc
            {
                // no ranges partially overlap
                for ka in hc.cache.keys().map(ra) {
                    for kb in hc.cache.keys().map(ra) {
                        let int = intersection(&ka, &kb);
                        if int.len() != 0 {
                            assert_eq!(int.len(), ka.len().min(kb.len()));
                        }
                    }
                }

                // all leaves are covered
                for i in 0..leaves.len() {
                    hc.cache.get(&(i, i + 1)).unwrap();
                }

                // no ranges are length zero
                for ka in hc.cache.keys().map(ra) {
                    assert_ne!(ka.len(), 0);
                }
            }
        }
    }

    fn range_of_lens() -> impl Iterator<Item = usize> {
        const START: usize = 1;
        const END: usize = 17 * 10;
        const STEP: usize = 17;
        (START..END).into_iter().step_by(STEP)
    }

    #[test]
    fn exercise_happy_path_sum() {
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
        happy_path::<Sum, _>(rando_ranges(range_of_lens()));
    }

    #[test]
    fn exercise_happy_path_() {
        struct DoubleThenSum;
        impl Merge for DoubleThenSum {
            type Hash = u8;
            fn leaf(a: &u8) -> u8 {
                a.wrapping_add(*a)
            }
            fn merge(a: &u8, b: &u8) -> u8 {
                a.wrapping_add(*b)
            }
        }
        happy_path::<DoubleThenSum, _>(rando_ranges(range_of_lens()));
    }

    #[test]
    fn exercise_happy_path_cat() {
        struct Cat;
        impl Merge for Cat {
            type Hash = String;
            fn leaf(a: &String) -> String {
                a.clone()
            }
            fn merge(a: &String, b: &String) -> String {
                format!("{}{}", a, b)
            }
        }
        happy_path::<Cat, _>(
            rando_ranges(range_of_lens())
                .map(|ns| ns.iter().map(|n: &usize| format!("{}", n)).collect()),
        );
    }

    #[test]
    fn exercise_happy_path_basic() {
        struct Basic;
        impl Merge for Basic {
            type Hash = [u8; 32];
            fn leaf(leaf: &Self::Hash) -> Self::Hash {
                leaf.clone()
            }
            fn merge(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
                let mut h = Blake2s::new();
                h.input(left);
                h.input(right);
                h.result().into()
            }
        }
        happy_path::<Basic, _>(rando_ranges(range_of_lens()));
    }

    #[test]
    fn exercise_happy_path_protected() {
        struct Protected;
        impl Merge for Protected {
            type Hash = [u8; 32];
            fn leaf(leaf: &Self::Hash) -> Self::Hash {
                Blake2s::digest(leaf).into()
            }
            fn merge(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
                let mut h = Blake2s::new();
                h.input(left);
                h.input(right);
                h.result().into()
            }
        }
        happy_path::<Protected, _>(rando_ranges(range_of_lens()));
    }

    fn rando_ranges<T>(r: impl IntoIterator<Item = usize>) -> impl Iterator<Item = Vec<T>>
    where
        Standard: Distribution<T>,
    {
        r.into_iter()
            .map(|len| -> Vec<T> { (0..len).into_iter().map(|_| rand::random()).collect() })
    }
}
