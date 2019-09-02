//! Block
use crate::codec::{Codec, ToBytes};
use crate::error::{format_err, Error};
use crate::hash::Hash;
use crate::ipld::Ipld;
pub use cid::Cid;
use std::convert::TryFrom;
use std::marker::PhantomData;

/// Raw block
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RawBlock {
    cid: Cid,
    data: Vec<u8>,
}

impl RawBlock {
    /// Creates a new `RawBlock`
    pub fn new(cid: Cid, data: Vec<u8>) -> Self {
        RawBlock { cid, data }
    }

    /// Returns the `CID` of the `Block`.
    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    /// Returns the data bytes of the `Block`.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    /// Takes a block apart.
    pub fn split(self) -> (Cid, Vec<u8>) {
        (self.cid, self.data)
    }
}

impl<TCodec, THash> From<Block<TCodec, THash>> for RawBlock {
    fn from(block: Block<TCodec, THash>) -> Self {
        block.to_raw()
    }
}

/// Block
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Block<TCodec, THash> {
    codec: PhantomData<TCodec>,
    hash: PhantomData<THash>,
    raw: RawBlock,
}

impl<TCodec, THash> Block<TCodec, THash> {
    /// Returns the raw block.
    pub fn raw(&self) -> &RawBlock {
        &self.raw
    }

    /// Returns the `CID` of the `Block`.
    pub fn cid(&self) -> &Cid {
        self.raw.cid()
    }

    /// Returns the data bytes of the `Block`.
    pub fn data(&self) -> &Vec<u8> {
        self.raw.data()
    }

    /// Takes a block apart.
    pub fn split(self) -> (Cid, Vec<u8>) {
        self.raw.split()
    }

    /// Returns the raw block.
    pub fn to_raw(self) -> RawBlock {
        self.raw
    }
}

impl<TCodec: Codec + ToBytes, THash> Block<TCodec, THash> {
    /// Returns the ipld of the block.
    pub fn ipld(&self) -> Result<Ipld, Error> {
        TCodec::from_bytes(self.data())
    }
}

impl<TCodec: Codec + ToBytes, THash: Hash> TryFrom<&Ipld> for Block<TCodec, THash> {
    type Error = Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        let data = TCodec::to_bytes(ipld)?;
        let hash = THash::digest(&data);
        let cid = Cid::new_v1(TCodec::CODEC, hash);
        Ok(Block {
            codec: PhantomData,
            hash: PhantomData,
            raw: RawBlock::new(cid, data),
        })
    }
}

impl<TCodec: Codec + ToBytes, THash: Hash> TryFrom<Ipld> for Block<TCodec, THash> {
    type Error = Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        Self::try_from(&ipld)
    }
}

impl<TCodec: Codec, THash: Hash> TryFrom<RawBlock> for Block<TCodec, THash> {
    type Error = failure::Error;

    fn try_from(raw: RawBlock) -> Result<Self, Self::Error> {
        if raw.cid().codec() != TCodec::CODEC {
            return Err(format_err!("Codec doesn't match"));
        }
        if raw.cid().hash().code() != THash::CODE {
            return Err(format_err!("Hash code doesn't match"));
        }
        if raw.cid().hash() != THash::digest(raw.data()).as_ref() {
            return Err(format_err!("Block hash does not match block data"));
        }
        Ok(Block {
            codec: PhantomData,
            hash: PhantomData,
            raw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cbor_block, codec, hash::Blake2b, ipld, json_block};

    #[test]
    fn test_block_from_ipld() {
        let block1 = cbor_block!({
            "metadata": {
                "type": "file",
                "name": "hello_world.txt",
                "size": 11,
            },
            "content": "hello world",
        })
        .unwrap();
        let block2 = json_block!({
            "metadata": {
                "type": "directory",
                "name": "folder",
                "size": 1,
            },
            "children": [
                block1.cid(),
            ]
        })
        .unwrap();
        let block3 = Block::<codec::DagJson, Blake2b>::try_from(block2.clone().to_raw()).unwrap();
        assert_eq!(block2, block3);

        let ipld = block3.ipld().unwrap();
        assert_eq!(
            ipld,
            ipld!({
                "metadata": {
                    "type": "directory",
                    "name": "folder",
                    "size": 1,
                },
                "children": [
                    block1.cid(),
                ]
            })
        );
    }
}
