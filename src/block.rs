//! Lazily encode/decode blocks.
use crate::cid::CidGeneric;
use crate::codec::IpldCodec;
use crate::encode_decode::EncodeDecodeIpld;
use crate::ipld::Ipld;
use crate::multihash::{Code as HCode, MultihashDigest};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

/// Block API Error.
#[derive(Error, Debug)]
pub enum BlockError {
    /// TDOO
    #[error("Cannot decode block.")]
    DecodeError,
    /// TDOO
    #[error("Cannot encode block.")]
    EncodeError,
    /// TDOO
    #[error("Cannot find codec implementation.")]
    CodecNotFound,
    /// TDOO
    #[error("Cannot find hash algorithm implementation.")]
    HashAlgNotFound,
}

/// Concrete Block which uses the default code tables
pub type Block = BlockGeneric<IpldCodec, HCode>;

/// Concrete Block with referenced data which uses the default code tables
pub type BlockRef<'a> = BlockRefGeneric<'a, IpldCodec, HCode>;

/// A `BlockGeneric` is an IPLD object together with a CID. The data can be encoded and decoded.
///
/// All operations are cached. This means that encoding, decoding and CID calculation happens
/// at most once. All subsequent calls will use a cached version.
pub struct BlockGeneric<C, H>
where
    C: Copy + TryFrom<u64> + Into<u64> + EncodeDecodeIpld<H>,
    H: Copy + TryFrom<u64> + Into<u64>,
{
    cid: Option<CidGeneric<C, H>>,
    raw: Option<Vec<u8>>,
    node: Option<Ipld<C, H>>,
    codec: C,
    hash_alg: H,
}

impl<C, H> BlockGeneric<C, H>
where
    C: Copy + TryFrom<u64> + Into<u64> + EncodeDecodeIpld<H>,
    H: Copy + TryFrom<u64> + Into<u64>,
{
    /// Create a new `Block` from the given CID and raw binary data.
    ///
    /// It needs a registry that contains codec and hash algorithms implementations in order to
    /// be able to decode the data into IPLD.
    pub fn new(cid: CidGeneric<C, H>, raw: Vec<u8>) -> Self {
        let codec = cid.codec();
        let hash_alg = cid.hash().algorithm();
        Self {
            cid: Some(cid),
            raw: Some(raw),
            node: None,
            codec,
            hash_alg,
        }
    }

    /// Create a new `Block` from the given IPLD object, codec and hash algorithm.
    ///
    /// No computation is done, the CID creation and the encoding will only be performed when the
    /// corresponding methods are called.
    pub fn encoder(node: Ipld<C, H>, codec: C, hash_alg: H) -> Self {
        Self {
            cid: None,
            raw: None,
            node: Some(node),
            codec,
            hash_alg,
        }
    }

    /// Create a new `Block` from encoded data, codec and hash algorithm.
    ///
    /// No computation is done, the CID creation and the decoding will only be performed when the
    /// corresponding methods are called.
    pub fn decoder(raw: Vec<u8>, codec: C, hash_alg: H) -> Self {
        Self {
            cid: None,
            raw: Some(raw),
            node: None,
            codec,
            hash_alg,
        }
    }

    /// Decode the `Block` into an IPLD object.
    ///
    /// The actual decoding is only performed if the object doesn't have a copy of the IPLD
    /// object yet. If that method was called before, it returns the cached result.
    pub fn decode(&mut self) -> Result<Ipld<C, H>, BlockError> {
        if let Some(node) = &self.node {
            Ok(node.clone())
        } else if let Some(raw) = &self.raw {
            //let decoded = Box::<dyn DynIpldCodec>::from(Box::new(self.codec))._decode(&raw).unwrap();
            let decoded = self.codec.decode(&raw).unwrap();
            self.node = Some(decoded.clone());
            Ok(decoded)
        } else {
            Err(BlockError::DecodeError)
        }
    }

    /// Encode the `Block` into raw binary data.
    ///
    /// The actual encoding is only performed if the object doesn't have a copy of the encoded
    /// raw binary data yet. If that method was called before, it returns the cached result.
    pub fn encode(&mut self) -> Result<Vec<u8>, BlockError> {
        if let Some(raw) = &self.raw {
            Ok(raw.clone())
        } else if let Some(node) = &self.node {
            //let encoded = Box::<dyn Codec>::from(self.codec)
            //    .encode(&node)
            //    .unwrap()
            //    .to_vec();
            let encoded = self.codec.encode(&node).unwrap().to_vec();
            self.raw = Some(encoded.clone());
            Ok(encoded)
        } else {
            Err(BlockError::EncodeError)
        }
    }

    /// Calculate the CID of the `Block`.
    ///
    /// The CID is calculated from the encoded data. If it wasn't encoded before, that
    /// operation will be performed. If the encoded data is already available, from a previous
    /// call of `encode()` or because the `Block` was instantiated via `encoder()`, then it
    /// isn't re-encoded.
    pub fn cid(&mut self) -> Result<CidGeneric<C, H>, BlockError>
    where
        Box<dyn MultihashDigest<H>>: From<H>,
    {
        if let Some(cid) = &self.cid {
            Ok(cid.clone())
        } else {
            // TODO vmx 2020-01-31: should probably be `encodeUnsafe()`
            let hash = Box::<dyn MultihashDigest<H>>::from(self.hash_alg).digest(&self.encode()?);
            let cid = CidGeneric::new_v1(self.codec, hash);
            Ok(cid)
        }
    }
}

impl<C, H> fmt::Debug for BlockGeneric<C, H>
where
    C: Copy + TryFrom<u64> + Into<u64> + fmt::Debug + EncodeDecodeIpld<H>,
    H: Copy + TryFrom<u64> + Into<u64> + fmt::Debug,
    H: Into<Box<dyn MultihashDigest<H>>>,
    <C as TryFrom<u64>>::Error: fmt::Debug,
    <H as TryFrom<u64>>::Error: fmt::Debug,
{
    fn fmt(&self, ff: &mut fmt::Formatter) -> fmt::Result {
        write!(ff, "Block {{ cid: {:?}, raw: {:?} }}", self.cid, self.raw)
    }
}

/// A `BlockRefGeneric` is an IPLD object together with a CID. The data can be encoded and decoded.
///
/// The `BlockRefGeneric` is very similar to the [`BlockGeneric`]. The difference is that it
/// doesn't own the data, but holds references only. When encoding/decoding of data is performed,
/// then the result is cached.
///
/// # Example
///
/// You want to get the CID from already encoded data. There no additional encoding will happen,
/// as we already have the data correctly encoded.
///
/// ```
/// use libipld::block::BlockRef;
/// use libipld::codec::IpldCodec;
/// use libipld::multihash::Code;
///
/// let encoded = &[0x11, 0x22];
/// let mut block = BlockRef::decoder(encoded, IpldCodec::Raw, Code::Sha2_256);
/// let cid = block.cid().unwrap();
/// ```
///
/// Additional encoding will happen in the following case, as only the decoded data is stored.
///
/// ```
/// use libipld::block::BlockRef;
/// use libipld::codec::IpldCodec;
/// use libipld::ipld::Ipld;
/// use libipld::multihash::Code;
///
/// let ipld = Ipld::Bool(true);
/// let mut block = BlockRef::encoder(&ipld, IpldCodec::DagCbor, Code::Sha2_256);
/// let cid = block.cid().unwrap();
/// // This call will return the cached CID as we already did create it once.
/// let cid2 = block.cid().unwrap();
/// ```
pub struct BlockRefGeneric<'a, C, H>
where
    C: Copy + TryFrom<u64> + Into<u64> + EncodeDecodeIpld<H>,
    H: Copy + TryFrom<u64> + Into<u64>,
{
    cid: Option<&'a CidGeneric<C, H>>,
    raw: Option<&'a [u8]>,
    node: Option<&'a Ipld<C, H>>,
    codec: C,
    hash_alg: H,
    cid_cached: Option<CidGeneric<C, H>>,
    raw_cached: Option<Vec<u8>>,
    node_cached: Option<Ipld<C, H>>,
}

impl<'a, C, H> BlockRefGeneric<'a, C, H>
where
    C: Copy + TryFrom<u64> + Into<u64> + EncodeDecodeIpld<H>,
    H: Copy + TryFrom<u64> + Into<u64>,
{
    /// Create a new `Block` from the given CID and raw binary data.
    ///
    /// It needs a registry that contains codec and hash algorithms implementations in order to
    /// be able to decode the data into IPLD.
    pub fn new(cid: &'a CidGeneric<C, H>, raw: &'a [u8]) -> Self {
        let codec = cid.codec();
        let hash_alg = cid.hash().algorithm();
        Self {
            cid: Some(cid),
            raw: Some(raw),
            node: None,
            codec,
            hash_alg,
            cid_cached: None,
            raw_cached: None,
            node_cached: None,
        }
    }

    /// Create a new `Block` from the given IPLD object, codec and hash algorithm.
    ///
    /// No computation is done, the CID creation and the encoding will only be performed when the
    /// corresponding methods are called.
    pub fn encoder(node: &'a Ipld<C, H>, codec: C, hash_alg: H) -> Self {
        Self {
            cid: None,
            raw: None,
            node: Some(node),
            codec,
            hash_alg,
            cid_cached: None,
            raw_cached: None,
            node_cached: None,
        }
    }

    /// Create a new `Block` from encoded data, codec and hash algorithm.
    ///
    /// No computation is done, the CID creation and the decoding will only be performed when the
    /// corresponding methods are called.
    pub fn decoder(raw: &'a [u8], codec: C, hash_alg: H) -> Self {
        Self {
            cid: None,
            raw: Some(raw),
            node: None,
            codec,
            hash_alg,
            cid_cached: None,
            raw_cached: None,
            node_cached: None,
        }
    }

    /// Decode the `Block` into an IPLD object.
    ///
    /// The actual decoding is only performed if the object doesn't have a copy of the IPLD
    /// object yet. If that method was called before, it returns the cached result.
    pub fn decode(&mut self) -> Result<Ipld<C, H>, BlockError> {
        // The block was constructed with a decdoded data
        if let Some(node) = &self.node {
            Ok((*node).clone())
        }
        // The decoder was already called at least once
        else if let Some(node_cached) = &self.node_cached {
            Ok(node_cached.clone())
        }
        // The data needs to be decoded
        else if let Some(raw) = &self.raw {
            let decoded = self.codec.decode(&raw).unwrap();
            self.node_cached = Some(decoded.clone());
            Ok(decoded)
        } else {
            Err(BlockError::DecodeError)
        }
    }

    /// Encode the `Block` into raw binary data.
    ///
    /// The actual encoding is only performed if the object doesn't have a copy of the encoded
    /// raw binary data yet. If that method was called before, it returns the cached result.
    pub fn encode(&mut self) -> Result<Vec<u8>, BlockError> {
        // The block was constructed with an encoded data
        if let Some(raw) = &self.raw {
            Ok(raw.to_vec())
        }
        // The encoder was already called at least once
        else if let Some(raw_cached) = &self.raw_cached {
            Ok(raw_cached.clone())
        }
        // The data needs to be decoded
        else if let Some(node) = &self.node {
            let encoded = self.codec.encode(&node).unwrap().to_vec();
            self.raw_cached = Some(encoded.clone());
            Ok(encoded)
        } else {
            Err(BlockError::EncodeError)
        }
    }

    /// Calculate the CID of the `Block`.
    ///
    /// The CID is calculated from the encoded data. If it wasn't encoded before, that
    /// operation will be performed. If the encoded data is already available, from a previous
    /// call of `encode()` or because the `Block` was instantiated via `encoder()`, then it
    /// isn't re-encoded.
    pub fn cid(&mut self) -> Result<CidGeneric<C, H>, BlockError>
    where
        Box<dyn MultihashDigest<H>>: From<H>,
    {
        // The block was constructed with a CID
        if let Some(cid) = &self.cid {
            Ok((*cid).clone())
        }
        // The CID was already calculated at leat once
        else if let Some(cid_cached) = &self.cid_cached {
            Ok(cid_cached.clone())
        }
        // The CID needs to be calculated
        else {
            let hasher = Box::<dyn MultihashDigest<H>>::from(self.hash_alg);
            // We already have the encoded data, use that one directly
            let hash = if let Some(raw) = &self.raw {
                hasher.digest(raw)
            } else {
                // TODO vmx 2020-01-31: should probably be `encodeUnsafe()`
                hasher.digest(&self.encode()?)
            };
            let cid = CidGeneric::new_v1(self.codec, hash);
            Ok(cid)
        }
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
    use crate::codec::Cid;
    use crate::codec::IpldCodec;
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
