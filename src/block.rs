//! Block
use crate::codec::{Codec, ToBytes};
use crate::hash::Hash;
use crate::untyped::Ipld;
use cid::{Cid, Prefix};
use std::marker::PhantomData;

/// Block
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Block<TCodec, THash> {
    codec: PhantomData<TCodec>,
    hash: PhantomData<THash>,
    cid: Cid,
    data: Vec<u8>,
}

impl<TCodec, THash> Block<TCodec, THash> {
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
            cid,
            data,
        }
    }
}
