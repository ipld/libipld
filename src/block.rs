//! Block validation
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::{Code, Multihasher};
use crate::raw::Raw;
use crate::MAX_BLOCK_SIZE;
#[cfg(feature = "dag-cbor")]
use dag_cbor::DagCbor;
#[cfg(feature = "dag-json")]
use dag_json::DagJson;
#[cfg(feature = "dag-pb")]
use dag_pb::DagPb;

/// Block
pub struct Block {
    /// Content identifier.
    pub cid: Cid,
    /// Binary data.
    pub data: Box<[u8]>,
}

/// Encode a block.
pub fn encode<C: Codec, H: Multihasher<Code>, E: Encode<C>>(e: &E) -> Result<Block> {
    let mut data = Vec::new();
    e.encode(&mut data)
        .map_err(|e| Error::CodecError(Box::new(e)))?;
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let hash = H::digest(&data);
    let cid = Cid::new_v1(C::CODE, hash);
    Ok(Block {
        cid,
        data: data.into_boxed_slice(),
    })
}

/// Decodes a block.
pub fn decode<C: Codec, D: Decode<C>>(cid: &Cid, mut data: &[u8]) -> Result<D> {
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let hash = cid.hash().algorithm().digest(data);
    if hash.as_ref() != cid.hash() {
        return Err(Error::InvalidHash(hash));
    }
    D::decode(&mut data).map_err(|e| Error::CodecError(Box::new(e)))
}

/// Decode block to ipld.
pub fn decode_ipld(cid: &Cid, _data: &[u8]) -> Result<Ipld> {
    match cid.codec() {
        Raw::CODE => decode::<Raw, _>(cid, _data),
        #[cfg(feature = "dag-cbor")]
        DagCbor::CODE => decode::<DagCbor, _>(cid, _data),
        #[cfg(feature = "dag-pb")]
        DagPb::CODE => decode::<DagPb, _>(cid, _data),
        #[cfg(feature = "dag-json")]
        DagJson::CODE => decode::<DagJson, _>(cid, _data),
        _ => Err(Error::UnsupportedCodec(cid.codec())),
    }
}
