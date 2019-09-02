//! Hash types.
use multihash::{Multihash, MultihashDigest};

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
