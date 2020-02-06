use super::merge::Merge;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProofElem<T> {
    Left(T),
    Right(T),
}

impl<T> ProofElem<T> {
    pub fn merge<M: Merge<Hash = T>>(&self, sibling: &M::Hash) -> M::Hash {
        match self {
            Self::Left(l) => M::merge(l, sibling),
            Self::Right(r) => M::merge(sibling, r),
        }
    }
}

impl<T> ProofElem<&T> {
    pub fn cloned(self) -> ProofElem<T>
    where
        T: Clone,
    {
        match self {
            Self::Left(l) => ProofElem::Left(l.clone()),
            Self::Right(r) => ProofElem::Left(r.clone()),
        }
    }
}
