//! Block validation
use crate::cid::Cid;
use crate::codec::{Code as CCode, Codec, Decode, Encode};
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::{Code as HCode, Multihasher};
use crate::raw::RawCodec;
use crate::MAX_BLOCK_SIZE;
#[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;
#[cfg(feature = "dag-json")]
use libipld_json::DagJsonCodec;
#[cfg(feature = "dag-pb")]
use libipld_pb::DagPbCodec;
use std::collections::HashSet;

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
        RawCodec::CODE => raw_decode::<RawCodec, _>(codec, data),
        #[cfg(feature = "dag-cbor")]
        DagCborCodec::CODE => raw_decode::<DagCborCodec, _>(codec, data),
        #[cfg(feature = "dag-pb")]
        DagPbCodec::CODE => raw_decode::<DagPbCodec, _>(codec, data),
        #[cfg(feature = "dag-json")]
        DagJsonCodec::CODE => raw_decode::<DagJsonCodec, _>(codec, data),
        _ => Err(Error::UnsupportedCodec(codec)),
    }
}

/// Decode block to ipld.
pub fn decode_ipld(cid: &Cid, data: &[u8]) -> Result<Ipld> {
    match cid.codec() {
        RawCodec::CODE => decode::<RawCodec, _>(cid, data),
        #[cfg(feature = "dag-cbor")]
        DagCborCodec::CODE => decode::<DagCborCodec, _>(cid, data),
        #[cfg(feature = "dag-pb")]
        DagPbCodec::CODE => decode::<DagPbCodec, _>(cid, data),
        #[cfg(feature = "dag-json")]
        DagJsonCodec::CODE => decode::<DagJsonCodec, _>(cid, data),
        _ => Err(Error::UnsupportedCodec(cid.codec())),
    }
}

/// Returns the references in an ipld block.
pub fn references(ipld: &Ipld) -> HashSet<Cid> {
    let mut set: HashSet<Cid> = Default::default();
    for ipld in ipld.iter() {
        if let Ipld::Link(cid) = ipld {
            set.insert(cid.to_owned());
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld;
    use crate::multihash::Sha2_256;

    #[test]
    fn test_references() {
        let cid1 = Cid::new_v0(Sha2_256::digest(b"cid1")).unwrap();
        let cid2 = Cid::new_v0(Sha2_256::digest(b"cid2")).unwrap();
        let cid3 = Cid::new_v0(Sha2_256::digest(b"cid3")).unwrap();
        let ipld = ipld!({
            "cid1": &cid1,
            "cid2": { "other": true, "cid2": { "cid2": &cid2 }},
            "cid3": [[ &cid3, &cid1 ]],
        });
        let refs = references(&ipld);
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&cid1));
        assert!(refs.contains(&cid2));
        assert!(refs.contains(&cid3));
    }
}
