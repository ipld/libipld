//! Traits for implementing a block store.
use crate::block::{Block, Cid};
use crate::codec::decode;
use crate::error::{format_err, Result};
use crate::hash::digest;
use crate::ipld::Ipld;

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
    fn read(&self, cid: &Cid) -> Result<Ipld>;
    /// Writes a raw block.
    fn write(&mut self, block: Block) -> Result<Cid>;
    /// Deletes the block with cid.
    fn delete(&mut self, cid: &Cid) -> Result<()>;
}

impl<T: BlockStore> IpldStore for T {
    fn read(&self, cid: &Cid) -> Result<Ipld> {
        let data = unsafe { BlockStore::read(self, cid)? };
        let hash = digest(cid.hash().code(), &data);
        if cid.hash() != hash.as_ref() {
            return Err(format_err!("Invalid data"));
        }
        let ipld = decode(cid.codec(), &data)?;
        Ok(ipld)
    }

    fn write(&mut self, block: Block) -> Result<Cid> {
        let (cid, data) = block.split();
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
