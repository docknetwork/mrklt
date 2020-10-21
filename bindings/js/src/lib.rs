use libmrkl::proof::ProofElem;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

/// Accepts a list of blake2s hashes packed into a single array. Returns the 32 byte root hash.
///
/// # Panics
///
/// Panics if the input value is not a multiple of 32.
#[wasm_bindgen]
pub fn compute_root(leaves: &[u8]) -> Box<[u8]> {
    let leaves = split32(leaves);
    let root = libmrkl::compute_root::<Blake2sSpr>(&leaves);
    Box::new(root)
}

/// # Panics
///
/// Panics if the packed "leaves" array length is not a multiple of 32.
#[wasm_bindgen]
pub fn create_proof(leaf_index: usize, leaves: &[u8]) -> JsValue {
    let leaves = split32(leaves);
    let proof = libmrkl::create_proof::<Blake2sSpr>(leaf_index, &leaves);
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
    let root = libmrkl::verify_proof::<Blake2sSpr>(&leaf, &proof);
    Box::new(root)
}

/// Blake2s Second Preimage Resisant
///
/// Blake2s hash with leaves double-hashed to resist second preimage attacks.
enum Blake2sSpr {}

impl libmrkl::Merge for Blake2sSpr {
    type Hash = [u8; 32];

    fn leaf(l: &[u8; 32]) -> [u8; 32] {
        // leaf is already hashed, but we hash it again resist SPA
        blake2s(&[l])
    }

    fn merge(l: &[u8; 32], r: &[u8; 32]) -> [u8; 32] {
        blake2s(&[l, r])
    }
}

fn blake2s(bs: &[&[u8]]) -> [u8; 32] {
    use blake2::{Blake2s, Digest};
    let mut hasher = Blake2s::new();
    for b in bs {
        hasher.update(&b);
    }
    to_fixed(&hasher.finalize())
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

// # Potential Optimization
//
// It would save cpu cycles to generate all proofs in a batch.
//
// ```
// #[wasm_bindgen]
// pub fn construct_tree(Vec<Leaves>) -> (Hash, Vec<Proof>);
// ```
//
// I'm going to resist the urge to optimize right now but an api of that form would reduce the
// required hashes for computing all proofs and the root.
//
// let `l` be the number of leaves.
//
// `(l-1) * (l+1)` is the current number of hashes required. Batching would change that number to
// `l-1`.
