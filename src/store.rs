//! Traits for implementing a block store.
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::codec::decode;
use crate::error::{format_err, Result};
use crate::hash::{digest, Hash};
use crate::ipld::{Cid, Ipld};
use cid::Codec;

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
    fn read_ipld(&self, cid: &Cid) -> Result<Ipld>;
    /// Reads the block with cid.
    fn read_cbor<C: ReadCbor>(&self, cid: &Cid) -> Result<C>;
    /// Writes a raw block.
    fn write_cbor<H: Hash, C: WriteCbor>(&mut self, c: &C) -> Result<Cid>;
    /// Deletes the block with cid.
    fn delete(&mut self, cid: &Cid) -> Result<()>;
}

impl<T: BlockStore> IpldStore for T {
    fn read_ipld(&self, cid: &Cid) -> Result<Ipld> {
        let data = unsafe { BlockStore::read(self, cid)? };
        let hash = digest(cid.hash().code(), &data);
        if cid.hash() != hash.as_ref() {
            return Err(format_err!("Invalid hash"));
        }
        decode(cid.codec(), &data)
    }

    fn read_cbor<C: ReadCbor>(&self, cid: &Cid) -> Result<C> {
        if cid.codec() != cid::Codec::DagCBOR {
            return Err(format_err!("Not cbor codec"));
        }
        let data = unsafe { BlockStore::read(self, cid)? };
        let hash = digest(cid.hash().code(), &data);
        if cid.hash() != hash.as_ref() {
            return Err(format_err!("Invalid hash"));
        }
        let mut data_ref: &[u8] = &data;
        C::read_cbor(&mut data_ref)
    }

    fn write_cbor<H: Hash, C: WriteCbor>(&mut self, c: &C) -> Result<Cid> {
        let mut data = Vec::new();
        c.write_cbor(&mut data)?;
        let hash = H::digest(&data);
        let cid = Cid::new_v1(Codec::DagCBOR, hash);
        unsafe { BlockStore::write(self, &cid, data.into_boxed_slice())? };
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
