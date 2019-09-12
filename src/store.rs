//! Traits for implementing a block store.
use crate::codec::{decode, Codec};
use crate::convert::{FromIpld, ToIpld};
use crate::error::{format_err, Result};
use crate::hash::{digest, Hash};
pub use cid::Cid;

/// The prefix of a block includes all information to serialize and deserialize
/// to/from ipld.
pub trait Prefix {
    /// The codec to use for encoding ipld.
    type Codec: Codec;
    /// The hash to use to compute the cid.
    type Hash: Hash;
}

/// Implementable by ipld storage backends.
pub trait BlockStore: Default {
    /// Returns the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    unsafe fn read(&self, cid: &Cid) -> Result<Box<[u8]>>;
    /// Writes the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    unsafe fn write(&mut self, cid: &Cid, data: Box<[u8]>) -> Result<()>;
    /// Deletes the block that match the cid.
    fn delete(&mut self, cid: &Cid) -> Result<()>;
}

/// Auto implemented trait for all block stores.
pub trait IpldStore: Default {
    /// Reads the block with cid.
    fn read<D: FromIpld>(&self, cid: &Cid) -> Result<D>;
    /// Writes a raw block.
    fn write<TPrefix: Prefix, S: ToIpld>(&mut self, s: &S) -> Result<Cid>;
    /// Deletes the block with cid.
    fn delete(&mut self, cid: &Cid) -> Result<()>;
}

impl<T: BlockStore> IpldStore for T {
    fn read<D: FromIpld>(&self, cid: &Cid) -> Result<D> {
        let data = unsafe { BlockStore::read(self, cid)? };
        let hash = digest(cid.hash().code(), &data);
        if cid.hash() != hash.as_ref() {
            return Err(format_err!("Invalid data"));
        }
        let ipld = decode(cid.codec(), &data)?;
        let d = D::from_ipld(ipld)?;
        Ok(d)
    }

    fn write<TPrefix: Prefix, S: ToIpld>(&mut self, s: &S) -> Result<Cid> {
        let data = TPrefix::Codec::encode(s.to_ipld())?;
        let hash = TPrefix::Hash::digest(&data);
        let cid = Cid::new_v1(TPrefix::Codec::CODEC, hash);
        unsafe { BlockStore::write(self, &cid, data)? };
        Ok(cid)
    }

    fn delete(&mut self, cid: &Cid) -> Result<()> {
        BlockStore::delete(self, cid)?;
        Ok(())
    }
}

pub mod mock {
    //! Utilities for testing
    use super::*;
    use multibase::Base;
    use std::collections::HashMap;

    /// A memory backed store
    pub type MemStore = HashMap<String, Box<[u8]>>;

    fn key(cid: &Cid) -> String {
        multibase::encode(Base::Base64UpperNoPad, cid.to_bytes())
    }

    impl BlockStore for MemStore {
        unsafe fn read(&self, cid: &Cid) -> Result<Box<[u8]>> {
            if let Some(data) = self.get(&key(cid)) {
                Ok(data.to_owned())
            } else {
                Err(format_err!("Block not found"))
            }
        }

        unsafe fn write(&mut self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
            self.insert(key(cid), data);
            Ok(())
        }

        fn delete(&mut self, cid: &Cid) -> Result<()> {
            self.remove(&key(cid));
            Ok(())
        }
    }
}
