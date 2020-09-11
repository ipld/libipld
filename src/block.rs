//! Block validation
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{BlockTooLarge, InvalidMultihash, Result, UnsupportedMultihash};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::store::StoreParams;
use core::borrow::Borrow;
use core::convert::TryFrom;
use core::marker::PhantomData;
use core::ops::Deref;
use std::collections::HashSet;

/// Block
#[derive(Clone)]
pub struct Block<S> {
    _marker: PhantomData<S>,
    /// Content identifier.
    cid: Cid,
    /// Binary data.
    data: Vec<u8>,
}

impl<S> core::fmt::Debug for Block<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Block")
            .field("cid", &self.cid)
            .field("data", &self.data)
            .finish()
    }
}

impl<S> Deref for Block<S> {
    type Target = Cid;

    fn deref(&self) -> &Self::Target {
        &self.cid
    }
}

impl<S> core::hash::Hash for Block<S> {
    fn hash<SH: core::hash::Hasher>(&self, hasher: &mut SH) {
        core::hash::Hash::hash(&self.cid, hasher)
    }
}

impl<S> PartialEq for Block<S> {
    fn eq(&self, other: &Self) -> bool {
        self.cid == other.cid
    }
}

impl<S> Eq for Block<S> {}

impl<S> Borrow<Cid> for Block<S> {
    fn borrow(&self) -> &Cid {
        &self.cid
    }
}

impl<S> AsRef<Cid> for Block<S> {
    fn as_ref(&self) -> &Cid {
        &self.cid
    }
}

impl<S> AsRef<[u8]> for Block<S> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
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

impl<S: StoreParams> Block<S> {
    /// Creates a new block. Returns an error if the hash doesn't match
    /// the data.
    pub fn new(cid: Cid, data: Vec<u8>) -> Result<Self> {
        verify_cid::<S::Hashes>(&cid, &data)?;
        Ok(Self::new_unchecked(cid, data))
    }

    /// Creates a new block without verifying the cid.
    pub fn new_unchecked(cid: Cid, data: Vec<u8>) -> Self {
        Self {
            _marker: PhantomData,
            cid,
            data,
        }
    }

    /// Returns the cid.
    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    /// Returns the payload.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the inner cid and data.
    pub fn into_inner(self) -> (Cid, Vec<u8>) {
        (self.cid, self.data)
    }

    /// Encode a block.`
    pub fn encode<CE: Codec, T: Encode<CE> + ?Sized>(
        codec: CE,
        hcode: u64,
        payload: &T,
    ) -> Result<Self>
    where
        CE: Into<S::Codecs>,
    {
        debug_assert_eq!(
            Into::<u64>::into(codec),
            Into::<u64>::into(Into::<S::Codecs>::into(codec))
        );
        let data = codec.encode(payload)?;
        if data.len() > S::MAX_BLOCK_SIZE {
            return Err(BlockTooLarge(data.len()).into());
        }
        let cid = create_cid::<S::Hashes>(codec.into(), hcode, &data)?;
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
    /// use libipld::store::DefaultStoreParams;
    ///
    /// let block =
    ///     Block::<DefaultStoreParams>::encode(DagCborCodec, SHA2_256, "Hello World!").unwrap();
    /// let ipld = block.decode::<DagCborCodec, Ipld>().unwrap();
    ///
    /// assert_eq!(ipld, Ipld::String("Hello World!".to_string()));
    /// ```
    pub fn decode<CD: Codec, T: Decode<CD>>(&self) -> Result<T>
    where
        S::Codecs: Into<CD>,
    {
        debug_assert_eq!(
            Into::<u64>::into(CD::try_from(self.cid.codec()).unwrap()),
            Into::<u64>::into(S::Codecs::try_from(self.cid.codec()).unwrap()),
        );
        verify_cid::<S::Hashes>(&self.cid, &self.data)?;
        CD::try_from(self.cid.codec())?.decode(&self.data)
    }

    /// Returns the decoded ipld.
    pub fn ipld(&self) -> Result<Ipld>
    where
        Ipld: Decode<S::Codecs>,
    {
        self.decode::<S::Codecs, Ipld>()
    }

    /// Returns the references.
    pub fn references(&self) -> Result<HashSet<Cid>>
    where
        Ipld: Decode<S::Codecs>,
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
    use crate::multihash::SHA2_256;
    use crate::store::DefaultStoreParams;

    type IpldBlock = Block<DefaultStoreParams>;

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
