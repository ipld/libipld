//! Hash types.
use crate::cid::Cid;
use core::hash::{BuildHasher, Hasher};
use multihash::{encode, Hash as Code, Multihash};
use std::collections::{HashMap, HashSet};

/// Trait for hash type markers.
pub trait Hash {
    /// The multihash code.
    const CODE: Code;

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
            const CODE: Code = Code::$name;

            fn digest(bytes: &[u8]) -> Multihash {
                encode(Self::CODE, bytes).unwrap()
            }
        }
    };
}

hash!(SHA1);
hash!(SHA2256);
hash!(SHA2512);
hash!(SHA3224);
hash!(SHA3256);
hash!(SHA3384);
hash!(SHA3512);
hash!(Keccak224);
hash!(Keccak256);
hash!(Keccak384);
hash!(Keccak512);
hash!(Blake2b256);
hash!(Blake2b512);
hash!(Blake2s128);
hash!(Blake2s256);

/// Compute digest of bytes.
pub fn digest(code: Code, bytes: &[u8]) -> Multihash {
    match code {
        Code::SHA1 => encode(Code::SHA1, bytes).unwrap(),
        Code::SHA2256 => encode(Code::SHA2256, bytes).unwrap(),
        Code::SHA2512 => encode(Code::SHA2512, bytes).unwrap(),
        Code::SHA3224 => encode(Code::SHA3224, bytes).unwrap(),
        Code::SHA3256 => encode(Code::SHA3256, bytes).unwrap(),
        Code::SHA3384 => encode(Code::SHA3384, bytes).unwrap(),
        Code::SHA3512 => encode(Code::SHA3512, bytes).unwrap(),
        Code::Keccak224 => encode(Code::Keccak224, bytes).unwrap(),
        Code::Keccak256 => encode(Code::Keccak256, bytes).unwrap(),
        Code::Keccak384 => encode(Code::Keccak384, bytes).unwrap(),
        Code::Keccak512 => encode(Code::Keccak512, bytes).unwrap(),
        Code::Blake2b256 => encode(Code::Blake2b256, bytes).unwrap(),
        Code::Blake2b512 => encode(Code::Blake2b512, bytes).unwrap(),
        Code::Blake2s128 => encode(Code::Blake2s128, bytes).unwrap(),
        Code::Blake2s256 => encode(Code::Blake2s256, bytes).unwrap(),
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
