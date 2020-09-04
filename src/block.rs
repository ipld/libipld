//! Block validation
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{InvalidMultihash, Result, UnsupportedMultihash};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use core::marker::PhantomData;
use std::collections::HashSet;

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
    pub fn encode<CE: Codec, T: Encode<CE> + ?Sized>(
        codec: CE,
        hcode: u64,
        payload: &T,
    ) -> Result<Self>
    where
        CE: Into<C>,
    {
        debug_assert_eq!(
            Into::<u64>::into(codec),
            Into::<u64>::into(Into::<C>::into(codec))
        );
        let data = codec.encode(payload)?;
        let cid = create_cid::<M>(codec.into(), hcode, &data)?;
        Ok(Self {
            _marker: PhantomData,
            cid,
            data,
        })
    }

    /// Decodes a block.
    ///
    /// # Example
    ///
    /// Decoding to [`Ipld`]:
    ///
    /// ```
    /// use libipld::block::Block;
    /// use libipld::cbor::DagCborCodec;
    /// use libipld::codec_impl::Multicodec;
    /// use libipld::ipld::Ipld;
    /// use libipld::multihash::{Multihash, SHA2_256};
    ///
    /// let block =
    ///     Block::<Multicodec, Multihash>::encode(DagCborCodec, SHA2_256, "Hello World!").unwrap();
    /// let ipld = block.decode::<DagCborCodec, Ipld>().unwrap();
    ///
    /// assert_eq!(ipld, Ipld::String("Hello World!".to_string()));
    /// ```
    pub fn decode<CD: Codec, T: Decode<CD>>(&self) -> Result<T>
    where
        C: Into<CD>,
    {
        debug_assert_eq!(
            Into::<u64>::into(CD::try_from(self.cid.codec()).unwrap()),
            Into::<u64>::into(C::try_from(self.cid.codec()).unwrap()),
        );
        verify_cid::<M>(&self.cid, &self.data)?;
        CD::try_from(self.cid.codec())?.decode(&self.data)
    }

    /// Returns the decoded ipld.
    pub fn ipld(&self) -> Result<Ipld>
    where
        Ipld: Decode<C>,
    {
        self.decode::<C, Ipld>()
    }

    /// Returns the references.
    pub fn references(&self) -> Result<HashSet<Cid>>
    where
        Ipld: Decode<C>,
    {
        Ok(self.ipld()?.references())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor::DagCborCodec;
    use crate::cid::DAG_CBOR;
    use crate::codec_impl::Multicodec;
    use crate::ipld;
    use crate::ipld::Ipld;
    use crate::multihash::{Multihash, SHA2_256};

    type IpldBlock = Block<Multicodec, Multihash>;

    #[test]
    fn test_references() {
        let b1 = IpldBlock::encode(Multicodec::Raw, SHA2_256, &ipld!(&b"cid1"[..])).unwrap();
        let b2 = IpldBlock::encode(Multicodec::DagJson, SHA2_256, &ipld!("cid2")).unwrap();
        let b3 = IpldBlock::encode(
            Multicodec::DagPb,
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
        let block = IpldBlock::encode(Multicodec::DagCbor, SHA2_256, &payload).unwrap();
        let payload2 = block.decode::<Multicodec, _>().unwrap();
        assert_eq!(payload, payload2);

        let refs = payload2.references();
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&b1.cid));
        assert!(refs.contains(&b2.cid));
        assert!(refs.contains(&b3.cid));
    }

    #[test]
    fn test_transmute() {
        let b1 = IpldBlock::encode(DagCborCodec, SHA2_256, &42).unwrap();
        assert_eq!(b1.cid.codec(), DAG_CBOR);
    }
}
