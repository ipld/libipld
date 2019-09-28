//! Hash types.
use crate::cid::Cid;
use core::hash::{BuildHasher, Hasher};
use multihash::{Multihash, MultihashDigest};
use std::collections::{HashMap, HashSet};

/// Trait for hash type markers.
pub trait Hash {
    /// The multihash code.
    const CODE: multihash::Code;

    /// Computes the multihash of a byte slice.
    fn digest(bytes: &[u8]) -> Multihash;
}

macro_rules! hash {
    ($name:ident) => {
        /// $name
        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        pub struct $name;

        #[allow(clippy::derive_hash_xor_eq)]
        impl Hash for $name {
            const CODE: multihash::Code = multihash::Code::$name;

            fn digest(bytes: &[u8]) -> Multihash {
                multihash::$name::digest(bytes)
            }
        }
    };
}

hash!(Sha1);
hash!(Sha2_256);
hash!(Sha2_512);
hash!(Sha3_224);
hash!(Sha3_256);
hash!(Sha3_384);
hash!(Sha3_512);
hash!(Keccak224);
hash!(Keccak256);
hash!(Keccak384);
hash!(Keccak512);
hash!(Blake2b);
hash!(Blake2s);
hash!(Murmur3_32);
hash!(Murmur3_128X64);

/// Compute digest of bytes.
pub fn digest(code: multihash::Code, bytes: &[u8]) -> Multihash {
    match code {
        multihash::Code::Sha1 => multihash::Sha1::digest(bytes),
        multihash::Code::Sha2_256 => multihash::Sha2_256::digest(bytes),
        multihash::Code::Sha2_512 => multihash::Sha2_512::digest(bytes),
        multihash::Code::Sha3_224 => multihash::Sha3_224::digest(bytes),
        multihash::Code::Sha3_256 => multihash::Sha3_256::digest(bytes),
        multihash::Code::Sha3_384 => multihash::Sha3_384::digest(bytes),
        multihash::Code::Sha3_512 => multihash::Sha3_512::digest(bytes),
        multihash::Code::Keccak224 => multihash::Keccak224::digest(bytes),
        multihash::Code::Keccak256 => multihash::Keccak256::digest(bytes),
        multihash::Code::Keccak384 => multihash::Keccak384::digest(bytes),
        multihash::Code::Keccak512 => multihash::Keccak512::digest(bytes),
        multihash::Code::Blake2b => multihash::Blake2b::digest(bytes),
        multihash::Code::Blake2s => multihash::Blake2s::digest(bytes),
        multihash::Code::Murmur3_32 => multihash::Murmur3_32::digest(bytes),
        multihash::Code::Murmur3_128X64 => multihash::Murmur3_128X64::digest(bytes),
    }
}

/// A hasher builder for cid hasher.
#[derive(Clone, Default)]
pub struct BuildCidHasher;

impl BuildHasher for BuildCidHasher {
    type Hasher = CidHasher;

    fn build_hasher(&self) -> Self::Hasher {
        CidHasher(None)
    }
}

/// A hasher that avoids rehashing cids by using the fact that they already
/// contain a hash.
pub struct CidHasher(Option<u64>);

impl Hasher for CidHasher {
    fn finish(&self) -> u64 {
        self.0.unwrap()
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!();
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = Some(i);
    }
}

/// A HashMap for Cid's
pub type CidHashMap<V> = HashMap<Cid, V, BuildCidHasher>;
/// A HashSet for Cid's
pub type CidHashSet = HashSet<Cid, BuildCidHasher>;
