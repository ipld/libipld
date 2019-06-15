//! Hash types.

/// Trait for hash type markers.
pub trait Hash {
    /// The multihash hasher.
    const HASH: multihash::Hash;
}

macro_rules! hash {
    ($name:ident) => {
        /// $name
        pub struct $name;

        impl Hash for $name {
            const HASH: multihash::Hash = multihash::Hash::$name;
        }
    };
}

hash!(SHA1);
hash!(SHA2256);
hash!(SHA2512);
hash!(SHA3512);
hash!(SHA3384);
hash!(SHA3256);
hash!(SHA3224);
