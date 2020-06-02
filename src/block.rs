//! Block validation
use crate::cid::CidGeneric;
use crate::codec::{Codec, Decode, Encode, IpldCodec};
use crate::encode_decode::EncodeDecodeIpld;
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::{Code as HCode, MultihashDigest, Multihasher};
use crate::MAX_BLOCK_SIZE;
use std::collections::HashSet;
use std::convert::TryFrom;

/// Block
pub struct Block<C = IpldCodec, H = HCode>
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
pub fn raw_decode_ipld<C, H>(codec: C, data: &[u8]) -> Result<Ipld<C, H>>
where
    C: Into<u64> + TryFrom<u64> + Copy + PartialEq + EncodeDecodeIpld<H>,
    H: Into<u64> + TryFrom<u64> + Copy + PartialEq,
{
    codec
        .decode(data)
        .map_err(|e| Error::CodecError(Box::new(e)))
}

/// Decode block to ipld.
pub fn decode_ipld<C, H>(cid: &CidGeneric<C, H>, data: &[u8]) -> Result<Ipld<C, H>>
where
    C: Into<u64> + TryFrom<u64> + Copy + PartialEq + EncodeDecodeIpld<H>,
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
    cid.codec()
        .decode(data)
        .map_err(|e| Error::CodecError(Box::new(e)))
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
    use crate::codec::Cid;
    use crate::ipld;
    use crate::multihash::Sha2_256;

    #[test]
    fn test_references() {
        let cid1 = Cid::new_v1(IpldCodec::Raw, Sha2_256::digest(b"cid1"));
        let cid2 = Cid::new_v1(IpldCodec::Raw, Sha2_256::digest(b"cid2"));
        let cid3 = Cid::new_v1(IpldCodec::Raw, Sha2_256::digest(b"cid3"));
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
