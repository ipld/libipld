//! Block
use crate::codec::{Codec, ToBytes};
use crate::hash::Hash;
use crate::untyped::Ipld;
use cid::{Cid, Prefix};
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
    pub fn into(self) -> (Cid, Vec<u8>) {
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
    /// Returns the `CID` of the `Block`.
    pub fn cid(&self) -> &Cid {
        self.raw.cid()
    }

    /// Returns the data bytes of the `Block`.
    pub fn data(&self) -> &Vec<u8> {
        self.raw.data()
    }

    /// Takes a block apart.
    pub fn into(self) -> (Cid, Vec<u8>) {
        self.raw.into()
    }

    /// Returns the raw block.
    pub fn to_raw(self) -> RawBlock {
        self.raw
    }
}

impl<TCodec: Codec + ToBytes, THash: Hash> From<&Ipld> for Block<TCodec, THash> {
    fn from(ipld: &Ipld) -> Self {
        let data = TCodec::to_bytes(ipld);
        let prefix = Prefix {
            version: TCodec::VERSION,
            codec: TCodec::CODEC,
            mh_type: THash::HASH,
            mh_len: THash::HASH.size() as usize,
        };
        let cid = Cid::new_from_prefix(&prefix, &data);
        Block {
            codec: PhantomData,
            hash: PhantomData,
            raw: RawBlock::new(cid, data),
        }
    }
}

impl<TCodec: Codec, THash: Hash> TryFrom<RawBlock> for Block<TCodec, THash> {
    type Error = failure::Error;

    fn try_from(raw: RawBlock) -> Result<Self, Self::Error> {
        let prefix = Prefix {
            version: TCodec::VERSION,
            codec: TCodec::CODEC,
            mh_type: THash::HASH,
            mh_len: THash::HASH.size() as usize,
        };
        if raw.cid().prefix() == prefix {
            Ok(Block {
                codec: PhantomData,
                hash: PhantomData,
                raw,
            })
        } else {
            Err(failure::format_err!("Prefix doesn't match"))
        }
    }
}
