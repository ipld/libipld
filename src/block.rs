//! Block validation
use crate::cid::{Cid, Codec as RawCodec};
use crate::error::{BlockError, IpldError};
use crate::hash::{digest, Hash};
use crate::ipld::Ipld;
use crate::MAX_BLOCK_SIZE;
use dag_cbor::{DagCborCodec, ReadCbor, WriteCbor};
#[cfg(feature = "dag-json")]
use dag_json::DagJsonCodec;
#[cfg(feature = "dag-pb")]
use dag_pb::DagPbCodec;

/// Validate block.
pub fn validate(cid: &Cid, data: &[u8]) -> Result<(), BlockError> {
    if data.len() > MAX_BLOCK_SIZE {
        return Err(BlockError::BlockTooLarge(data.len()));
    }
    let hash = digest(cid.hash().algorithm(), &data)?;
    if hash.as_ref() != cid.hash() {
        return Err(BlockError::InvalidHash(hash));
    }
    Ok(())
}

/// Create raw block.
pub fn create_raw_block<H: Hash>(data: Box<[u8]>) -> Result<(Cid, Box<[u8]>), BlockError> {
    if data.len() > MAX_BLOCK_SIZE {
        return Err(BlockError::BlockTooLarge(data.len()));
    }
    let hash = H::digest(&data);
    let cid = Cid::new_v1(RawCodec::Raw, hash);
    Ok((cid, data))
}

/// Create cbor block.
pub fn create_cbor_block<H: Hash, C: WriteCbor>(c: &C) -> Result<(Cid, Box<[u8]>), BlockError> {
    let mut data = Vec::new();
    c.write_cbor(&mut data)?;
    if data.len() > MAX_BLOCK_SIZE {
        return Err(BlockError::BlockTooLarge(data.len()));
    }
    let hash = H::digest(&data);
    let cid = Cid::new_v1(DagCborCodec::CODEC, hash);
    Ok((cid, data.into_boxed_slice()))
}

/// Encode ipld to bytes.
pub fn encode_ipld(ipld: &Ipld, codec: RawCodec) -> Result<Box<[u8]>, BlockError> {
    let bytes = match codec {
        DagCborCodec::CODEC => DagCborCodec::encode(ipld)?,
        #[cfg(feature = "dag-pb")]
        DagPbCodec::CODEC => DagPbCodec::encode(ipld)?,
        #[cfg(feature = "dag-json")]
        DagJsonCodec::CODEC => DagJsonCodec::encode(ipld)?,
        RawCodec::Raw => {
            if let Ipld::Bytes(bytes) = ipld {
                bytes.to_vec().into_boxed_slice()
            } else {
                return Err(BlockError::CodecError(IpldError::NotBytes.into()));
            }
        }
        _ => return Err(BlockError::UnsupportedCodec(codec)),
    };
    Ok(bytes)
}

/// Decode block to ipld.
pub fn decode_ipld(cid: &Cid, data: &[u8]) -> Result<Ipld, BlockError> {
    let ipld = match cid.codec() {
        DagCborCodec::CODEC => DagCborCodec::decode(data)?,
        #[cfg(feature = "dag-pb")]
        DagPbCodec::CODEC => DagPbCodec::decode(data)?,
        #[cfg(feature = "dag-json")]
        DagJsonCodec::CODEC => DagJsonCodec::decode(data)?,
        RawCodec::Raw => Ipld::Bytes(data.to_vec()),
        _ => return Err(BlockError::UnsupportedCodec(cid.codec())),
    };
    Ok(ipld)
}

/// Decode block from cbor.
pub fn decode_cbor<C: ReadCbor>(cid: &Cid, mut data: &[u8]) -> Result<C, BlockError> {
    if cid.codec() != DagCborCodec::CODEC {
        return Err(BlockError::UnsupportedCodec(cid.codec()));
    }
    let res = C::read_cbor(&mut data)?;
    Ok(res)
}
