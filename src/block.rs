//! Block validation
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{InvalidMultihash, Result, UnsupportedMultihash};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use core::marker::PhantomData;

/// Block
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block<C, M> {
    _marker: PhantomData<(C, M)>,
    /// Content identifier.
    pub cid: Cid,
    /// Binary data.
    pub data: Box<[u8]>,
}

// TODO: move to tiny_cid
fn create_cid<M: MultihashDigest>(ccode: u64, hcode: u64, payload: &[u8]) -> Result<Cid> {
    let digest = M::new(hcode, payload)
        .map_err(|_| UnsupportedMultihash(hcode))?
        .to_raw()
        .map_err(|_| UnsupportedMultihash(hcode))?;
    Ok(Cid::new_v1(ccode, digest))
}

// TODO: move to tiny_cid
fn verify_cid<M: MultihashDigest>(cid: &Cid, payload: &[u8]) -> Result<()> {
    let hcode = cid.hash().code();
    let mh = M::new(hcode, payload).map_err(|_| UnsupportedMultihash(hcode))?;
    if mh.digest() != cid.hash().digest() {
        return Err(InvalidMultihash(mh.to_bytes()).into());
    }
    Ok(())
}

impl<C: Codec, M: MultihashDigest> Block<C, M> {
    /// Creates a new block.
    pub fn new(cid: Cid, data: Box<[u8]>) -> Self {
        Self {
            _marker: PhantomData,
            cid,
            data,
        }
    }

    /// Encode a block.`
    pub fn encode<T: Encode<C>>(ccode: u64, hcode: u64, payload: &T) -> Result<Self> {
        let data = C::try_from(ccode)?.encode(payload)?;
        let cid = create_cid::<M>(ccode, hcode, &data)?;
        Ok(Self {
            _marker: PhantomData,
            cid,
            data,
        })
    }

    /// Encode ipld.
    pub fn encode_ipld(ccode: u64, hcode: u64, ipld: &Ipld) -> Result<Self> {
        let data = C::try_from(ccode)?.encode_ipld(ipld)?;
        let cid = create_cid::<M>(ccode, hcode, &data)?;
        Ok(Self {
            _marker: PhantomData,
            cid,
            data,
        })
    }

    /// Decodes a block.
    pub fn decode<T: Decode<C>>(&self) -> Result<T> {
        verify_cid::<M>(&self.cid, &self.data)?;
        C::try_from(self.cid.codec())?.decode(&mut &self.data[..])
    }

    /// Decodes to ipld.
    pub fn decode_ipld(&self) -> Result<Ipld> {
        verify_cid::<M>(&self.cid, &self.data)?;
        C::try_from(self.cid.codec())?.decode_ipld(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cid::{DAG_CBOR, DAG_JSON, DAG_PROTOBUF, RAW};
    use crate::codec_impl::IpldCodec;
    use crate::ipld;
    use crate::multihash::{Multihash, SHA2_256};

    type IpldBlock = Block<IpldCodec, Multihash>;

    #[test]
    fn test_references() {
        let b1 = IpldBlock::encode(RAW, SHA2_256, &ipld!(&b"cid1"[..])).unwrap();
        let b2 = IpldBlock::encode(DAG_JSON, SHA2_256, &ipld!("cid2")).unwrap();
        let b3 = IpldBlock::encode(
            DAG_PROTOBUF,
            SHA2_256,
            &ipld!({
                "Data": &b"data"[..],
                "Links": Ipld::List(vec![]),
            }),
        )
        .unwrap();

        let payload = ipld!({
            "cid1": &b1.cid,
            "cid2": { "other": true, "cid2": { "cid2": &b2.cid }},
            "cid3": [[ &b3.cid, &b1.cid ]],
        });
        let block = IpldBlock::encode(DAG_CBOR, SHA2_256, &payload).unwrap();
        let payload2 = block.decode().unwrap();
        assert_eq!(payload, payload2);

        let refs = payload2.references();
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&b1.cid));
        assert!(refs.contains(&b2.cid));
        assert!(refs.contains(&b3.cid));
    }
}
