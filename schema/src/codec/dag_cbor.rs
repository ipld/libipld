use super::Error;
use crate::dev::*;
use crate::error::Error::Block;
use libipld_base::error::BlockError;
use serde::de::{Error as DError, Visitor};
use serde_cbor::{
    from_slice,
    tags::{current_cbor_tag, Tagged},
    to_vec, Error as CborError,
};
use std::{convert::TryFrom, fmt};

pub const CBOR_LINK_TAG: u64 = 42;
pub struct DagCbor;

#[async_trait]
impl Codec for DagCbor {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    type Error = Error;

    /// Encode function.
    ///
    /// TODO: impl `Encode` and `Serialize` for `Ipld`
    async fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error> {
        unimplemented!()
    }
    /// Decode function.
    ///
    /// TODO: impl `Decode` and `Deserialize` for `Ipld`
    async fn decode(data: &[u8]) -> Result<Ipld, Self::Error> {
        unimplemented!()
    }
}

impl CodecExt for DagCbor {
    fn encode<S>(dag: &S) -> Result<Box<[u8]>, Self::Error>
    where
        S: Serialize,
    {
        Ok(to_vec(dag)?.into())
    }

    fn decode<'de, D>(bytes: &'de [u8]) -> Result<D, Self::Error>
    where
        D: Deserialize<'de>,
    {
        Ok(from_slice(bytes)?)
    }

    fn serialize_link<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<u8> = cid.to_bytes();
        let bytes: &[u8] = vec.as_ref();
        Tagged::new(Some(CBOR_LINK_TAG), bytes).serialize(serializer)
    }

    fn deserialize_unknown<'de, D, V>(deserializer: D, visitor: V) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: IpldVisitor<'de>,
    {
        let visitor = DagCborLinkVisitor(visitor);
        visitor.visit_newtype_struct(deserializer)
    }
}

/// Helper visitor for deserializing links.
struct DagCborLinkVisitor<V>(V);
impl<'de, V> Visitor<'de> for DagCborLinkVisitor<V>
where
    V: IpldVisitor<'de>,
{
    type Value = V::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an IPLD link")
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match current_cbor_tag() {
            Some(CBOR_LINK_TAG) => {
                let bytes = <&[u8]>::deserialize(deserializer)?;
                let cid = Cid::try_from(bytes).or(Err(DError::custom("expected IPLD link")))?;
                self.0.visit_link(cid)
            }
            Some(tag) => Err(DError::custom(format!("unexpected tag: {}", tag))),
            _ => Err(DError::custom("tag expected")),
        }
    }
}

impl From<CborError> for Error {
    fn from(err: CborError) -> Self {
        Error::Codec(err.into())
    }
}
