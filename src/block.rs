//! Lazily encode/decode blocks.
use crate::cid::CidGeneric;
use crate::codec::{Codec, Decode, Encode, IpldCodec};
use crate::encode_decode::EncodeDecodeIpld;
use crate::ipld::Ipld;
use crate::multihash::{Code as HCode, MultihashDigest, Multihasher};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

/// Block API Error.
#[derive(Error, Debug)]
pub enum BlockError {
    /// Block cannot be decoded.
    #[error("Cannot decode block.")]
    DecodeError(String),
    /// Block cannot be encoded.
    #[error("Cannot encode block.")]
    EncodeError(String),
    /// Codec is not supported.
    #[error("Codec `{0} is not supported.")]
    UnsupportedCodec(u64),
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
    raw: Option<Box<[u8]>>,
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
    pub fn new(cid: CidGeneric<C, H>, raw: Box<[u8]>) -> Self {
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
    pub fn decoder(raw: Box<[u8]>, codec: C, hash_alg: H) -> Self {
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
    pub fn decode(&mut self) -> Result<&Ipld<C, H>, BlockError> {
        if let Some(ref node) = self.node {
            Ok(node)
        } else if let Some(ref raw) = self.raw {
            let decoded = self
                .codec
                .decode(raw)
                .map_err(|err| BlockError::DecodeError(err.to_string()))?;
            self.node = Some(decoded);
            Ok(self.node.as_ref().unwrap())
        } else {
            Err(BlockError::DecodeError(
                "Block is missing data.".to_string(),
            ))
        }
    }

    /// Encode the `Block` into raw binary data.
    ///
    /// The actual encoding is only performed if the object doesn't have a copy of the encoded
    /// raw binary data yet. If that method was called before, it returns the cached result.
    pub fn encode(&mut self) -> Result<&[u8], BlockError> {
        if let Some(ref raw) = self.raw {
            Ok(raw)
        } else if let Some(ref node) = self.node {
            let encoded = self
                .codec
                .encode(node)
                .map_err(|err| BlockError::EncodeError(err.to_string()))?;
            self.raw = Some(encoded);
            Ok(self.raw.as_ref().unwrap())
        } else {
            Err(BlockError::EncodeError("Block is missing data".to_string()))
        }
    }

    /// Calculate the CID of the `Block`.
    ///
    /// The CID is calculated from the encoded data. If it wasn't encoded before, that
    /// operation will be performed. If the encoded data is already available, from a previous
    /// call of `encode()` or because the `Block` was instantiated via `encoder()`, then it
    /// isn't re-encoded.
    pub fn cid(&mut self) -> Result<&CidGeneric<C, H>, BlockError>
    where
        Box<dyn MultihashDigest<H>>: From<H>,
    {
        if let Some(ref cid) = self.cid {
            Ok(cid)
        } else {
            let hash = Box::<dyn MultihashDigest<H>>::from(self.hash_alg).digest(&self.encode()?);
            self.cid = Some(CidGeneric::new_v1(self.codec, hash));
            Ok(self.cid.as_ref().unwrap())
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
    raw_cached: Option<Box<[u8]>>,
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
    pub fn decode(&mut self) -> Result<&Ipld<C, H>, BlockError> {
        // The block was constructed with a decdoded data
        if let Some(node) = self.node {
            Ok(node)
        }
        // The decoder was already called at least once
        else if let Some(ref node_cached) = self.node_cached {
            Ok(node_cached)
        }
        // The data needs to be decoded
        else if let Some(ref raw) = self.raw {
            let decoded = self
                .codec
                .decode(raw)
                .map_err(|err| BlockError::DecodeError(err.to_string()))?;
            self.node_cached = Some(decoded);
            Ok(self.node_cached.as_ref().unwrap())
        } else {
            Err(BlockError::DecodeError(
                "Block is missing data.".to_string(),
            ))
        }
    }

    /// Encode the `Block` into raw binary data.
    ///
    /// The actual encoding is only performed if the object doesn't have a copy of the encoded
    /// raw binary data yet. If that method was called before, it returns the cached result.
    pub fn encode(&mut self) -> Result<&[u8], BlockError> {
        // The block was constructed with an encoded data
        if let Some(raw) = self.raw {
            Ok(raw)
        }
        // The encoder was already called at least once
        else if let Some(ref raw_cached) = self.raw_cached {
            Ok(raw_cached)
        }
        // The data needs to be decoded
        else if let Some(ref node) = self.node {
            let encoded = self
                .codec
                .encode(node)
                .map_err(|err| BlockError::EncodeError(err.to_string()))?;
            self.raw_cached = Some(encoded);
            Ok(self.raw_cached.as_ref().unwrap())
        } else {
            Err(BlockError::EncodeError("Block is missing data".to_string()))
        }
    }

    /// Calculate the CID of the `Block`.
    ///
    /// The CID is calculated from the encoded data. If it wasn't encoded before, that
    /// operation will be performed. If the encoded data is already available, from a previous
    /// call of `encode()` or because the `Block` was instantiated via `encoder()`, then it
    /// isn't re-encoded.
    pub fn cid(&mut self) -> Result<&CidGeneric<C, H>, BlockError>
    where
        Box<dyn MultihashDigest<H>>: From<H>,
    {
        // The block was constructed with a CID
        if let Some(cid) = self.cid {
            Ok(cid)
        }
        // The CID was already calculated at leat once
        else if let Some(ref cid_cached) = self.cid_cached {
            Ok(cid_cached)
        }
        // The CID needs to be calculated
        else {
            let hasher = Box::<dyn MultihashDigest<H>>::from(self.hash_alg);
            // We already have the encoded data, use that one directly
            let hash = if let Some(raw) = &self.raw {
                hasher.digest(raw)
            } else {
                hasher.digest(&self.encode()?)
            };
            self.cid_cached = Some(CidGeneric::new_v1(self.codec, hash));
            Ok(self.cid_cached.as_ref().unwrap())
        }
    }
}

/// Encode native types into a block.
///
/// # Example
///
/// ```
/// use libipld::block::encode;
/// use libipld::cbor::DagCborCodec;
/// use libipld::multihash::{Code as HCode, Sha2_256};
/// use libipld::IpldCodec;
///
/// let native = "Hello World!".to_string();
/// let mut block = encode::<IpldCodec, HCode, DagCborCodec, Sha2_256, _>(&native).unwrap();
/// assert_eq!(
///     block.cid().unwrap().to_string(),
///     "bafyreih62zarvnosx5aktyzkhk6ufn5b33eqmm5te5ozor25r3rfigznje"
/// );
/// ```
pub fn encode<C, H, O, M, E>(e: &E) -> Result<BlockGeneric<C, H>, BlockError>
where
    O: Codec<C>,
    M: Multihasher<H>,
    E: Encode<O, C>,
    C: Into<u64> + TryFrom<u64> + Copy + EncodeDecodeIpld<H>,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    let mut data = Vec::new();
    e.encode(&mut data)
        .map_err(|err| BlockError::EncodeError(err.to_string()))?;
    let hash = M::digest(&data);
    let cid = CidGeneric::<C, H>::new_v1(O::CODE, hash);
    Ok(BlockGeneric::new(cid, data.into_boxed_slice()))
}

/// Decode into native types.
///
/// Useful for nested encodings when for example the data is encrypted.
///
/// # Example
///
/// ```
/// use libipld::block::decode;
/// use libipld::cbor::DagCborCodec;
/// use libipld::multihash::{Code as HCode, Sha2_256};
/// use libipld::IpldCodec;
///
/// let data = [
///     0x6c, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
/// ];
/// let native: String =
///     decode::<IpldCodec, HCode, DagCborCodec, _>(IpldCodec::DagCbor, &data).unwrap();
/// assert_eq!(native, "Hello World!".to_string());
/// ```
pub fn decode<C, H, O, D>(codec: C, mut data: &[u8]) -> Result<D, BlockError>
where
    O: Codec<C>,
    D: Decode<O, C>,
    C: Into<u64> + TryFrom<u64> + Copy + PartialEq,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    if codec != O::CODE {
        return Err(BlockError::UnsupportedCodec(codec.into()));
    }
    D::decode(&mut data).map_err(|err| BlockError::DecodeError(err.to_string()))
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
    use crate::cbor::DagCborCodec;
    use crate::codec::{Cid, IpldCodec};
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

    #[test]
    fn test_encode() {
        let native = "Hello World!".to_string();
        let mut block = encode::<IpldCodec, HCode, DagCborCodec, Sha2_256, _>(&native).unwrap();

        let mut block_ipld =
            Block::encoder(Ipld::String(native), IpldCodec::DagCbor, HCode::Sha2_256);
        assert_eq!(block.cid().unwrap(), block_ipld.cid().unwrap());
    }

    #[test]
    fn test_decode() {
        let ipld = Ipld::String("Hello World!".to_string());
        let mut block = Block::encoder(ipld, IpldCodec::DagCbor, HCode::Sha2_256);
        let data = block.encode().unwrap();

        let native: String =
            decode::<IpldCodec, HCode, DagCborCodec, _>(IpldCodec::DagCbor, &data).unwrap();
        assert_eq!(native, "Hello World!".to_string());
    }
}
