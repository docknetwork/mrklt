/// This trait can be implemented to allow, or to disallow second preimage attacks.
///
/// ```rust
/// use libmrkl::Merge;
///
/// impl Merge for BlockSecondPreimage {
///     type Hash = [u8; 32];
///
///     fn leaf(leaf: &Self::Hash) -> Self::Hash {
///         hash(leaf)
///     }
///
///     fn merge(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
///         cat_hash(left, right)
///     }
/// }
///
/// impl Merge for AllowSecondPreimage {
/// # type Hash = ();
/// # /*
///     ...
/// # */
///     fn leaf(leaf: &Self::Hash) -> Self::Hash {
///         leaf.clone()
///     }
/// # /*
///     ...
/// # */
/// # fn merge(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
/// #     unimplemented!()
/// # }
/// }
///
/// // Prepending 0x00 or 0x01 as described in [rfc6962](https://www.rfc-editor.org/info/rfc6962).
/// impl Merge for AlternativeBlockSecondPreimage {
///     type Hash = Box<[u8]>;
///
///     fn leaf(leaf: &Self::Hash) -> Self::Hash {
///         let mut ret = Vec::<u8>::with_capacity(1 + leaf.len());
///         ret.push(0x00);
///         ret.extend_from_slice(leaf);
///         ret.into_boxed_slice()
///     }
///
///     fn merge(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
///         let mut ret = Vec::<u8>::from(cat_hash(left, right));
///         ret.insert(0, 0x01);
///         ret.into_boxed_slice()
///     }
/// }
/// #
/// # struct AllowSecondPreimage;
/// # struct BlockSecondPreimage;
/// # struct AlternativeBlockSecondPreimage;
/// #
/// # fn hash<A: Any>(a: &A) -> A {
/// #     unimplemented!()
/// # }
/// #
/// # fn cat_hash<A: Any>(a: &A, b: &A) -> A {
/// #     unimplemented!()
/// # }
/// # use core::any::Any;
/// ```
pub trait Merge {
    type Hash;

    /// Compute the hash of a leaf node.
    ///
    /// Note:
    ///   The leaf passed has already been hashed once. An implementations may modify the leaf
    ///   somehow. Before inclusion.
    fn leaf(leaf: &Self::Hash) -> Self::Hash;

    /// Compute the hash of an inner node.
    fn merge(left: &Self::Hash, right: &Self::Hash) -> Self::Hash;
}
