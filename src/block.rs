//! Block validation
use crate::cid::{Cid, CidGeneric};
use crate::codec::{Code as CCode, Codec, Decode, Encode};
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::{Code as HCode, MultihashDigest, Multihasher};
use crate::raw::RawCodec;
use crate::MAX_BLOCK_SIZE;
#[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;
#[cfg(feature = "dag-json")]
use libipld_json::DagJsonCodec;
#[cfg(feature = "dag-pb")]
use libipld_pb::DagPbCodec;
use std::collections::HashSet;
use std::convert::TryFrom;

/// Block
pub struct Block<C = CCode, H = HCode>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    /// Content identifier.
    pub cid: CidGeneric<C, H>,
    /// Binary data.
    pub data: Box<[u8]>,
}

/// Encode a block.`
pub fn encode<C, H, O, M, E>(e: &E) -> Result<Block<C, H>>
where
    O: Codec<C>,
    M: Multihasher<H>,
    E: Encode<O, C>,
    C: Into<u64> + TryFrom<u64> + Copy,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    let mut data = Vec::new();
    e.encode(&mut data)
        .map_err(|e| Error::CodecError(Box::new(e)))?;
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let hash = M::digest(&data);
    let cid = CidGeneric::<C, H>::new_v1(O::CODE, hash);
    Ok(Block {
        cid,
        data: data.into_boxed_slice(),
    })
}

/// Raw decode.
///
/// Useful for nested encodings when for example the data is encrypted.
pub fn raw_decode<C, H, O, D>(codec: C, mut data: &[u8]) -> Result<D>
where
    O: Codec<C>,
    D: Decode<O, C>,
    C: Into<u64> + TryFrom<u64> + Copy + PartialEq,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    if u64::try_from(codec).unwrap() != O::CODE.into() {
        return Err(Error::UnsupportedCodec(codec.into()));
    }
    D::decode(&mut data).map_err(|e| Error::CodecError(Box::new(e)))
}

/// Decodes a block.
pub fn decode<C, H, O, D>(cid: &CidGeneric<C, H>, data: &[u8]) -> Result<D>
where
    O: Codec<C>,
    D: Decode<O, C>,
    C: Into<u64> + TryFrom<u64> + Copy + PartialEq,
    H: Into<u64> + TryFrom<u64> + Copy + PartialEq,
    Box<dyn MultihashDigest<H>>: From<H>,
{
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let hash = Box::<dyn MultihashDigest<H>>::from(cid.hash().algorithm()).digest(&data);
    if hash.as_ref() != cid.hash() {
        return Err(Error::InvalidHash(hash.to_vec()));
    }
    raw_decode::<C, H, O, D>(cid.codec(), data)
}

/// Decode raw ipld.
///
/// Useful for nested encodings when for example the data is encrypted.
pub fn raw_decode_ipld(codec: CCode, data: &[u8]) -> Result<Ipld> {
    match codec {
        RawCodec::CODE => raw_decode::<CCode, HCode, RawCodec, _>(codec, data),
        #[cfg(feature = "dag-cbor")]
        DagCborCodec::CODE => raw_decode::<CCode, HCode, DagCborCodec, _>(codec, data),
        #[cfg(feature = "dag-pb")]
        DagPbCodec::CODE => raw_decode::<CCode, HCode, DagPbCodec, _>(codec, data),
        #[cfg(feature = "dag-json")]
        DagJsonCodec::CODE => raw_decode::<CCode, HCode, DagJsonCodec, _>(codec, data),
        _ => Err(Error::UnsupportedCodec(codec.into())),
    }
}

/// Decode block to ipld.
pub fn decode_ipld(cid: &Cid, data: &[u8]) -> Result<Ipld> {
    match cid.codec() {
        RawCodec::CODE => decode::<CCode, HCode, RawCodec, _>(cid, data),
        #[cfg(feature = "dag-cbor")]
        DagCborCodec::CODE => decode::<CCode, HCode, DagCborCodec, _>(cid, data),
        #[cfg(feature = "dag-pb")]
        DagPbCodec::CODE => decode::<CCode, HCode, DagPbCodec, _>(cid, data),
        #[cfg(feature = "dag-json")]
        DagJsonCodec::CODE => decode::<CCode, HCode, DagJsonCodec, _>(cid, data),
        _ => Err(Error::UnsupportedCodec(cid.codec().into())),
    }
}

/// Returns the references in an ipld block.
pub fn references<C, H>(ipld: &Ipld<C, H>) -> HashSet<CidGeneric<C, H>>
where
    C: Into<u64> + TryFrom<u64> + Copy + Eq,
    H: Into<u64> + TryFrom<u64> + Copy + Eq,
{
    let mut set: HashSet<CidGeneric<C, H>> = Default::default();
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
    use crate::cid::Cid;
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
