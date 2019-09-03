//! Block
use crate::codec::{Codec, IpldCodec, ToBytes};
use crate::error::{format_err, Error, Result};
use crate::hash::Hash;
use crate::ipld::Ipld;
pub use cid::Cid as RawCid;
use core::convert::TryFrom;
use multihash::Multihash;
use std::marker::PhantomData;

/// The prefix of a block includes all information to serialize and deserialize
///  to/from ipld.
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
    unsafe fn read(&self, cid: &RawCid) -> Result<Box<[u8]>>;
    /// Writes the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    unsafe fn write(&mut self, cid: &RawCid, data: Box<[u8]>) -> Result<()>;
    /// Deletes the block that match the cid.
    fn delete(&mut self, cid: &RawCid) -> Result<()>;
}

/// Auto implemented trait for all block stores.
pub trait IpldStore: Default {
    /// Reads the block with cid.
    fn read<TPrefix: Prefix>(
        &self,
        cid: &Cid<TPrefix>,
    ) -> Result<Block<TPrefix>>;
    /// Writes a raw block.
    fn write<TPrefix: Prefix>(
        &mut self,
        block: RawBlock<TPrefix>,
    ) -> Result<Cid<TPrefix>>;
    /// Deletes the block with cid.
    fn delete<TPrefix: Prefix>(&mut self, cid: &Cid<TPrefix>) -> Result<()>;
}

impl<T: BlockStore> IpldStore for T {
    fn read<TPrefix: Prefix>(
        &self,
        cid: &Cid<TPrefix>,
    ) -> Result<Block<TPrefix>> {
        let data = unsafe { BlockStore::read(self, cid.raw())? };
        let raw = RawBlock::<TPrefix>::new(data);
        if raw.cid().raw() != cid.raw() {
            return Err(format_err!("Invalid data"));
        }
        Ok(Block::try_from(raw)?)
    }

    fn write<TPrefix: Prefix>(
        &mut self,
        block: RawBlock<TPrefix>,
    ) -> Result<Cid<TPrefix>> {
        let RawBlock { cid, data } = block;
        unsafe { BlockStore::write(self, cid.raw(), data)? };
        Ok(cid)
    }

    fn delete<TPrefix: Prefix>(&mut self, cid: &Cid<TPrefix>) -> Result<()> {
        BlockStore::delete(self, cid.raw())?;
        Ok(())
    }
}

/// Cid
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Cid<TPrefix: Prefix> {
    prefix: PhantomData<TPrefix>,
    raw: RawCid,
}

impl<TPrefix: Prefix> Cid<TPrefix> {
    /// Creates a new cid from a hash.
    pub fn new(hash: Multihash) -> Self {
        Self {
            prefix: PhantomData,
            raw: RawCid::new_v1(TPrefix::Codec::CODEC, hash),
        }
    }

    /// Returns the raw cid.
    pub fn raw(&self) -> &RawCid {
        &self.raw
    }
}

impl<TPrefix: Prefix> From<Cid<TPrefix>> for RawCid {
    fn from(cid: Cid<TPrefix>) -> Self {
        cid.raw
    }
}

impl<TPrefix: Prefix> From<&Cid<TPrefix>> for Ipld {
    fn from(cid: &Cid<TPrefix>) -> Self {
        cid.raw().into()
    }
}

impl<TPrefix: Prefix> TryFrom<RawCid> for Cid<TPrefix> {
    type Error = Error;

    fn try_from(cid: RawCid) -> Result<Self> {
        if cid.codec() != TPrefix::Codec::CODEC {
            return Err(format_err!("Codec doesn't match"));
        }
        if cid.hash().code() != TPrefix::Hash::CODE {
            return Err(format_err!("Hash code doesn't match"));
        }
        Ok(Self {
            prefix: PhantomData,
            raw: cid,
        })
    }
}

/// RawBlock
#[derive(Clone, Debug, PartialEq)]
pub struct RawBlock<TPrefix: Prefix> {
    cid: Cid<TPrefix>,
    data: Box<[u8]>,
}

impl<TPrefix: Prefix> RawBlock<TPrefix> {
    /// Creates a raw block from binary data.
    pub fn new(data: Box<[u8]>) -> Self {
        let hash = TPrefix::Hash::digest(&data);
        let cid = Cid::new(hash);
        Self { cid, data }
    }

    /// Returns the cid of the block.
    pub fn cid(&self) -> &Cid<TPrefix> {
        &self.cid
    }
}

/// Block
#[derive(Clone, Debug, PartialEq)]
pub struct Block<TPrefix: Prefix> {
    prefix: PhantomData<TPrefix>,
    ipld: Ipld,
}

impl<TPrefix: Prefix> Block<TPrefix> {
    /// Creates a block from ipld.
    pub fn new(ipld: Ipld) -> Self {
        Self {
            prefix: PhantomData,
            ipld,
        }
    }

    /// Returns a reference to the ipld.
    pub fn ipld(&self) -> &Ipld {
        &self.ipld
    }

    /// Returns a mutable reference to the ipld.
    pub fn ipld_mut(&mut self) -> &mut Ipld {
        &mut self.ipld
    }

    /// Returns the raw block.
    pub fn to_raw(&self) -> Result<RawBlock<TPrefix>> {
        let data = TPrefix::Codec::to_bytes(self.ipld())?;
        Ok(RawBlock::new(data))
    }
}

impl<TPrefix: Prefix> TryFrom<RawBlock<TPrefix>> for Block<TPrefix> {
    type Error = Error;

    fn try_from(raw: RawBlock<TPrefix>) -> Result<Self> {
        Ok(Block {
            prefix: raw.cid.prefix,
            ipld: TPrefix::Codec::from_bytes(&raw.data)?,
        })
    }
}

impl<TPrefix: Prefix> TryFrom<Block<TPrefix>> for RawBlock<TPrefix> {
    type Error = Error;

    fn try_from(block: Block<TPrefix>) -> Result<Self> {
        block.to_raw()
    }
}

impl<TPrefix: Prefix> From<Ipld> for Block<TPrefix> {
    fn from(ipld: Ipld) -> Self {
        Self::new(ipld)
    }
}

impl<TPrefix: Prefix> From<Block<TPrefix>> for Ipld {
    fn from(block: Block<TPrefix>) -> Self {
        block.ipld
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block;
    use std::collections::HashMap;

    type Store = HashMap<String, Box<[u8]>>;

    impl BlockStore for Store {
        unsafe fn read(&self, cid: &RawCid) -> Result<Box<[u8]>> {
            if let Some(data) = self.get(&cid.to_string()) {
                Ok(data.to_owned())
            } else {
                Err(format_err!("Block not found"))
            }
        }

        unsafe fn write(&mut self, cid: &RawCid, data: Box<[u8]>) -> Result<()> {
            self.insert(cid.to_string(), data);
            Ok(())
        }

        fn delete(&mut self, cid: &RawCid) -> Result<()> {
            self.remove(&cid.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_block() {
        let block1 = block!({
            "metadata": {
                "type": "file",
                "name": "hello_world.txt",
                "size": 11,
            },
            "content": "hello world",
        })
        .to_raw()
        .unwrap();
        block!({
            "metadata": {
                "type": "directory",
                "name": "folder",
                "size": 1,
            },
            "children": [ block1.cid() ],
        })
        .to_raw()
        .unwrap();
    }
}
