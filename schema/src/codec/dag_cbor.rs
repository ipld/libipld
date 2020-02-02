use crate::{dev::*, Error};
use libipld_base::error::BlockError;
use serde::de::{Error as DError, Visitor};
use serde_cbor::{
    from_reader, from_slice,
    tags::{current_cbor_tag, Tagged},
    to_vec, to_writer, Error as CborError,
};
use std::{
    convert::TryFrom,
    fmt,
    io::{Read, Write},
};

/// The magic tag
pub const CBOR_LINK_TAG: u64 = 42;

/// The DagCBOR codec, that delegates to `serde_cbor`.
pub struct DagCbor;

#[async_trait]
impl Codec for DagCbor {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    type Error = Error;

    /// TODO: impl `Encode` and `Serialize` for `Ipld`
    async fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error> {
        unimplemented!()
    }

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

    fn write<S, W>(dag: &S, writer: W) -> Result<(), Self::Error>
    where
        S: Serialize,
        W: Write,
    {
        Ok(to_writer(writer, dag)?)
    }

    fn read<D, R>(reader: R) -> Result<D, Self::Error>
    where
        D: DeserializeOwned,
        R: Read,
    {
        Ok(from_reader(reader)?)
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
