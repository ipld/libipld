//! Block validation
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{Error, Result};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::MAX_BLOCK_SIZE;
use std::collections::HashSet;
use core::marker::PhantomData;
use core::convert::{TryInto, TryFrom};

/// Block
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block<C, M, T> {
    _marker: PhantomData<(C, M, T)>,
    /// Content identifier.
    pub cid: Cid,
    /// Binary data.
    pub data: Box<[u8]>,
}

impl<C, M, T> Block<C, M, T>
where
    C: Codec + TryFrom<u64, Error = Error>,
    M: MultihashDigest,
    T: Encode<C>,
{
    /// Encode a block.`
    pub fn encode(ccode: u64, hcode: u64, payload: &T) -> Result<Self> {
        let mut bytes = Vec::with_capacity(MAX_BLOCK_SIZE);
        payload.encode(ccode.try_into()?, &mut bytes)
            .map_err(|e| Error::CodecError(Box::new(e)))?;
        if bytes.len() > MAX_BLOCK_SIZE {
            return Err(Error::BlockTooLarge(bytes.len()));
        }
        let digest = M::new(hcode, &bytes)
            .map_err(|_| Error::UnsupportedMultihash(hcode))?
            .to_raw()
            .map_err(|_| Error::UnsupportedMultihash(hcode))?;
        let cid = Cid::new_v1(ccode, digest);
        Ok(Self {
            _marker: PhantomData,
            cid,
            data: bytes.into_boxed_slice(),
        })
    }
}

impl<C, M, T> Block<C, M, T>
where
    C: Codec + TryFrom<u64, Error = Error>,
    M: MultihashDigest,
    T: Decode<C>,
{
    /// Decodes a block.
    pub fn decode(&self) -> Result<T> {
        if self.data.len() > MAX_BLOCK_SIZE {
            return Err(Error::BlockTooLarge(self.data.len()));
        }
        let mh = M::new(self.cid.hash().code(), &self.data)
            .map_err(|_| Error::UnsupportedMultihash(self.cid.hash().code()))?;
        if mh.digest() != self.cid.hash().digest() {
            return Err(Error::InvalidHash(mh.to_bytes()));
        }
        T::decode(self.cid.codec().try_into()?, &mut &self.data[..])
            .map_err(|e| Error::CodecError(Box::new(e)))
    }
}

impl<C, M, T> Block<C, M, T>
where
    C: Codec + TryFrom<u64, Error = Error>,
{
    /// Returns the references in an ipld block.
    pub fn references(&self) -> Result<HashSet<Cid>> {
        let ipld = Ipld::decode(self.cid.codec().try_into()?, &mut &self.data[..])
            .map_err(|e| Error::CodecError(Box::new(e)))?;
        let mut set: HashSet<Cid> = Default::default();
        for ipld in ipld.iter() {
            if let Ipld::Link(cid) = ipld {
                set.insert(cid.to_owned());
            }
        }
        Ok(set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cid::{DAG_CBOR, DAG_JSON, DAG_PROTOBUF, RAW};
    use crate::codec_impl::IpldCodec;
    use crate::ipld;
    use crate::multihash::{Multihash, SHA2_256};

    type IpldBlock = Block<IpldCodec, Multihash, Ipld>;

    #[test]
    fn test_references() {
        let b1 = IpldBlock::encode(RAW, SHA2_256, &ipld!(&b"cid1"[..])).unwrap();
        let b2 = IpldBlock::encode(DAG_JSON, SHA2_256, &ipld!("cid2")).unwrap();
        let b3 = IpldBlock::encode(DAG_PROTOBUF, SHA2_256, &ipld!({
            "Data": &b"data"[..],
            "Links": Ipld::List(vec![]),
        })).unwrap();

        let payload = ipld!({
            "cid1": &b1.cid,
            "cid2": { "other": true, "cid2": { "cid2": &b2.cid }},
            "cid3": [[ &b3.cid, &b1.cid ]],
        });
        let block = IpldBlock::encode(DAG_CBOR, SHA2_256, &payload).unwrap();
        let payload2 = block.decode().unwrap();
        assert_eq!(payload, payload2);

        let refs = block.references();
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&b1.cid));
        assert!(refs.contains(&b2.cid));
        assert!(refs.contains(&b3.cid));
    }
}
