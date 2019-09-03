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
        let ipld = decode(cid.codec(), data)?;
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

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;

    pub type Store = HashMap<String, Box<[u8]>>;

    impl BlockStore for Store {
        unsafe fn read(&self, cid: &Cid) -> Result<Box<[u8]>> {
            if let Some(data) = self.get(&cid.to_string()) {
                Ok(data.to_owned())
            } else {
                Err(format_err!("Block not found"))
            }
        }

        unsafe fn write(&mut self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
            self.insert(cid.to_string(), data);
            Ok(())
        }

        fn delete(&mut self, cid: &Cid) -> Result<()> {
            self.remove(&cid.to_string());
            Ok(())
        }
    }
}
