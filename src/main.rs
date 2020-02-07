mod proofelem;

use core::str::FromStr;
use mrkl;
use proofelem::Elem;
use std::string::ToString;
use structopt::StructOpt;

enum HashAlg {
    Blake2,
    Groestl,
    Ripemd160,
    Ripemd320,
    Sha2,
    Sha3,
    Whirlpool,
}

impl FromStr for HashAlg {
    type Err = &'static str;
    fn from_str(a: &str) -> Result<Self, &'static str> {
        Ok(match a {
            "blake2" => Self::Blake2,
            "groestl" => Self::Groestl,
            "ripemd160" => Self::Ripemd160,
            "ripemd320" => Self::Ripemd320,
            "sha2" => Self::Sha2,
            "sha3" => Self::Sha3,
            "whirlpool" => Self::Whirlpool,
            _ => Err(
                "alg must be one of: blake2, groestl, ripemd160, ripemd320, sha2, sha3, whirlpool",
            )?,
        })
    }
}

#[derive(StructOpt)]
struct Args {
    /// one of: blake2, groestl, ripemd160, ripemd320, sha2, sha3, whirlpool
    alg: HashAlg,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Root(Root),
    Verify(Verify),
    Proof(Proof),
}

#[derive(StructOpt)]
/// Compute a merkle root.
struct Root {
    /// Ordered list of leaves.
    leaves: Vec<Hash>,
}

#[derive(StructOpt)]
/// Verify a proof.
struct Verify {
    /// Leaf hash we want to verify.
    leaf: Hash,
    /// Merkle root.
    root: Hash,
    /// "l" or "r" prefixed proof elements.
    proof: Vec<Elem<Hash>>,
}

#[derive(StructOpt)]
/// Create a proof
struct Proof {
    /// Index of the leaf for which an inclusion proof will be generated
    index: usize,
    /// Original list of leaves. Required for proof generation.
    leaves: Vec<Hash>,
}

#[derive(Debug)]
struct Hash(Vec<u8>);

impl FromStr for Hash {
    type Err = hex::FromHexError;
    fn from_str(a: &str) -> Result<Self, hex::FromHexError> {
        hex::decode(a).map(Self)
    }
}

impl ToString for Hash {
    fn to_string(&self) -> String {
        hex::encode(&self.0)
    }
}

fn main() {
    Args::from_args();
}
