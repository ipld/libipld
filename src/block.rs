//! Block validation
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::MAX_BLOCK_SIZE;
use std::collections::HashSet;

/// Block
#[derive(Clone, Debug, Eq, PartialEq)]
    /// Content identifier.
    pub cid: Cid,
    /// Binary data.
    pub data: Box<[u8]>,
}

/// Encode a block.`
pub fn encode<C, E, M>(codec: u64, hash: u64, e: &E) -> Result<Block>
where
    C: Codec,
    E: Encode<C>,
    M: MultihashDigest,
{
    let mut data = Vec::with_capacity(MAX_BLOCK_SIZE);
    e.encode(&mut data)
        .map_err(|e| Error::CodecError(Box::new(e)))?;
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let hash = M::new(hash, &data)
        .map_err(|_| Error::UnsupportedMultihash(hash))?
        .to_raw()
        .map_err(|_| Error::UnsupportedMultihash(hash))?;
    let cid = Cid::new_v1(codec, hash);
    Ok(Block {
        cid,
        data: data.into_boxed_slice(),
    })
}

/// Decodes a block.
pub fn decode<C, D, M>(cid: &Cid, data: &[u8]) -> Result<D>
where
    C: Codec,
    D: Decode<C>,
    M: MultihashDigest,
{
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let mh = M::new(cid.hash().code(), &data)
        .map_err(|_| Error::UnsupportedMultihash(cid.hash().code()))?;
    if mh.digest() != cid.hash().digest() {
        return Err(Error::InvalidHash(mh.to_bytes()));
    }
    D::decode(&mut data).map_err(|e| Error::CodecError(Box::new(e)))
}

/// Decode block to ipld.
pub fn decode_ipld<M: MultihashDigest>(cid: &Cid, data: &[u8]) -> Result<Ipld> {
    if data.len() > MAX_BLOCK_SIZE {
        return Err(Error::BlockTooLarge(data.len()));
    }
    let mh = M::new(cid.hash().code(), data)
        .map_err(|_| Error::UnsupportedMultihash(cid.hash().code()))?;
    if mh.digest() != cid.hash().digest() {
        return Err(Error::InvalidHash(mh.to_bytes()));
    }
    cid.codec()
        .decode(data)
        .map_err(|e| Error::CodecError(Box::new(e)))
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
