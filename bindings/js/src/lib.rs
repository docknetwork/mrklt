use mrklt::proof::ProofElem;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

/// Accepts a list of blake2b-256 hashes packed into a single array. Returns the 32 byte root hash.
///
/// # Panics
///
/// Panics if the input value is not a multiple of 32.
/// Panics if number of leaves is 0.
#[wasm_bindgen]
pub fn compute_root(leaves: &[u8]) -> Box<[u8]> {
    let leaves = split32(leaves);
    let root = mrklt::compute_root::<Blake2b256Spr>(&leaves);
    Box::new(root)
}

/// # Panics
///
/// Panics if the packed "leaves" array length is not a multiple of 32.
/// Panics if leaf_index is >= leaves.len().
#[wasm_bindgen]
pub fn create_proof(leaf_index: usize, leaves: &[u8]) -> JsValue {
    let leaves = split32(leaves);
    let proof = mrklt::create_proof::<Blake2b256Spr>(leaf_index, &leaves);
    JsValue::from_serde(&proof).unwrap()
}

/// Returns the 32 byte root hash.
///
/// # Panics
///
/// Panics if leaf has a length other than 32.
/// Panics if proof is not a valid json serialized list of proof elements.
#[wasm_bindgen]
pub fn verify_proof(leaf: &[u8], proof: JsValue) -> Box<[u8]> {
    let leaf = to_fixed(leaf);
    let proof: Box<[ProofElem<[u8; 32]>]> = proof.into_serde().unwrap();
    let root = mrklt::verify_proof::<Blake2b256Spr>(&leaf, &proof);
    Box::new(root)
}

/// Compute root and create proofs for every leaf. This is much more efficient than calling
/// [`compute_root`] followed by [`create_proof`] for every element of the tree.
///
/// Accepts a list of blake2b-256 hashes packed into a single array. Returns the 32 byte root hash
/// and a list of proofs as a tuple.
///
/// # Panics
///
/// Panics if the input value is not a multiple of 32.
/// Panics if number of leaves is 0.
#[wasm_bindgen]
pub fn construct(leaves: &[u8]) -> JsValue {
    let leaves = split32(leaves);
    let a = mrklt::proof_map::HashCache::from_leaves::<Blake2b256Spr>(&leaves);
    let proofs: Vec<Box<[ProofElem<[u8; 32]>]>> = leaves
        .iter()
        .enumerate()
        .map(|(i, _)| a.create_proof(i))
        .collect();
    JsValue::from_serde(&(a.root(), proofs)).expect("serialization of return value failed")
}

/// Blake2b256 Second Preimage Resisant
///
/// Blake2b256 hash with leaves double-hashed to resist second preimage attacks.
enum Blake2b256Spr {}

impl mrklt::Merge for Blake2b256Spr {
    type Hash = [u8; 32];

    fn leaf(l: &[u8; 32]) -> [u8; 32] {
        // leaf is already hashed, but we hash it again resist SPA
        blake2b256(&[l])
    }

    fn merge(l: &[u8; 32], r: &[u8; 32]) -> [u8; 32] {
        blake2b256(&[l, r])
    }
}

fn blake2b256(bs: &[&[u8]]) -> [u8; 32] {
    use blake2::{
        digest::{Update, VariableOutput},
        VarBlake2b,
    };
    let mut hasher = VarBlake2b::new(32).unwrap();
    for b in bs {
        hasher.update(&b);
    }
    let mut ret = [0u8; 32];
    hasher.finalize_variable(|digest| ret = to_fixed(digest));
    ret
}

/// Panics if bs is not 32 bytes long.
fn to_fixed(bs: &[u8]) -> [u8; 32] {
    let mut ret = [0u8; 32];
    ret.copy_from_slice(bs);
    ret
}

// This could probably be done with unsafe and core::mem::transmute but let's wait for it to be a
// problem before making the optimization. Also, I haven't finished reading the nomicon so I don't
// have my unsafe license.
//
// The zero-allocations version would look something like this:
// ```
// fn split<'a>(bs: &'a [u8]) -> &'a [[u8; 32]];
// ```
// and would panic or if length is not a multiple of 32.
//
/// # Panics
///
/// Panics if length of input is not a multiple of 32.
fn split32(bs: &[u8]) -> Vec<[u8; 32]> {
    assert!(
        bs.len() % 32 == 0,
        "invalid length for packed 32 byte elements"
    );
    bs.chunks(32).map(to_fixed).collect()
}
