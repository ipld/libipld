//! Block validation
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::codec::{Codec, DagCborCodec};
use crate::error::BlockError;
use crate::hash::{digest, Hash};
use crate::ipld::{Cid, Ipld};
use crate::MAX_BLOCK_SIZE;
use multibase::Base;
use std::path::{Path, PathBuf};

/// Validate block.
pub fn validate(cid: &Cid, data: &[u8]) -> Result<(), BlockError> {
    if data.len() > MAX_BLOCK_SIZE {
        return Err(BlockError::BlockToLarge(data.len()));
    }
    let hash = digest(cid.hash().code(), &data);
    if hash.as_ref() != cid.hash() {
        return Err(BlockError::InvalidHash);
    }
    Ok(())
}

/// Path for block.
pub fn locate(store: &Box<Path>, cid: &Cid) -> Box<Path> {
    let base64 = multibase::encode(Base::Base64UrlUpperNoPad, cid.to_bytes());
    let mut buf = PathBuf::from(store.clone());
    buf.push(base64);
    buf.into_boxed_path()
}

/// Create cbor block.
pub async fn create_cbor_block<H: Hash, C: WriteCbor>(c: &C) -> Result<(Cid, Box<[u8]>), BlockError> {
    let mut data = Vec::new();
    c.write_cbor(&mut data).await
        .map_err(|e| BlockError::CodecError(e))?;
    if data.len() > MAX_BLOCK_SIZE {
        return Err(BlockError::BlockToLarge(data.len()));
    }
    let hash = H::digest(&data);
    let cid = Cid::new_v1(DagCborCodec::CODEC, hash);
    Ok((cid, data.into_boxed_slice()))
}

/// Decode block to ipld.
pub async fn decode_ipld(cid: &Cid, data: &[u8]) -> Result<Ipld, BlockError> {
    match cid.codec() {
        DagCborCodec::CODEC => DagCborCodec::decode(&data).await.map_err(|e| BlockError::CodecError(e)),
        _ => Err(BlockError::UnsupportedCodec(cid.codec())),
    }
}

/// Decode block from cbor.
pub async fn decode_cbor<C: ReadCbor + Send>(cid: &Cid, mut data: &[u8]) -> Result<C, BlockError> {
    if cid.codec() != DagCborCodec::CODEC {
        return Err(BlockError::UnsupportedCodec(cid.codec()))
    }
    C::read_cbor(&mut data).await.map_err(|e| BlockError::CodecError(e))
}
