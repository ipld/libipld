//! Block validation
use crate::cid::Cid;
use crate::codec::{Code as CCode, Codec, Decode, Encode};
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::{Code as HCode, Multihasher};
use crate::raw::Raw;
use crate::MAX_BLOCK_SIZE;
#[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCbor;
#[cfg(feature = "dag-json")]
use libipld_json::DagJson;
#[cfg(feature = "dag-pb")]
use libipld_pb::DagPb;

/// Block
pub struct Block {
    /// Content identifier.
    pub cid: Cid,
    /// Binary data.
    pub data: Box<[u8]>,
}

/// Encode a block.
pub fn encode<C: Codec, H: Multihasher<HCode>, E: Encode<C>>(e: &E) -> Result<Block> {
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

/// Raw decode.
///
/// Useful for nested encodings when for example the data is encrypted.
pub fn raw_decode<C: Codec, D: Decode<C>>(codec: CCode, mut data: &[u8]) -> Result<D> {
    if codec != C::CODE {
        return Err(Error::UnsupportedCodec(codec));
    }
    D::decode(&mut data).map_err(|e| Error::CodecError(Box::new(e)))
}

/// Decodes a block.
pub fn decode<C: Codec, D: Decode<C>>(cid: &Cid, data: &[u8]) -> Result<D> {
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let hash = cid.hash().algorithm().digest(data);
    if hash.as_ref() != cid.hash() {
        return Err(Error::InvalidHash(hash));
    }
    raw_decode(cid.codec(), data)
}

/// Decode raw ipld.
///
/// Useful for nested encodings when for example the data is encrypted.
pub fn raw_decode_ipld(codec: CCode, data: &[u8]) -> Result<Ipld> {
    match codec {
        Raw::CODE => raw_decode::<Raw, _>(codec, data),
        #[cfg(feature = "dag-cbor")]
        DagCbor::CODE => raw_decode::<DagCbor, _>(codec, data),
        #[cfg(feature = "dag-pb")]
        DagPb::CODE => raw_decode::<DagPb, _>(codec, data),
        #[cfg(feature = "dag-json")]
        DagJson::CODE => raw_decode::<DagJson, _>(codec, data),
        _ => Err(Error::UnsupportedCodec(codec)),
    }
}

/// Decode block to ipld.
pub fn decode_ipld(cid: &Cid, data: &[u8]) -> Result<Ipld> {
    match cid.codec() {
        Raw::CODE => decode::<Raw, _>(cid, data),
        #[cfg(feature = "dag-cbor")]
        DagCbor::CODE => decode::<DagCbor, _>(cid, data),
        #[cfg(feature = "dag-pb")]
        DagPb::CODE => decode::<DagPb, _>(cid, data),
        #[cfg(feature = "dag-json")]
        DagJson::CODE => decode::<DagJson, _>(cid, data),
        _ => Err(Error::UnsupportedCodec(cid.codec())),
    }
}
