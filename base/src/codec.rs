//! `Ipld` codecs.
use crate::error::BlockError;
use crate::ipld::Ipld;
use async_trait::async_trait;
use cid::Cid;
use core::fmt::Debug;
use failure::Fail;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{self, Visitor}};

/// Codec trait.
#[async_trait]
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Error type.
    type Error: Debug + Fail + Into<BlockError>;
    /// Encode function.
    async fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error>;
    /// Decode function.
    async fn decode(data: &[u8]) -> Result<Ipld, Self::Error>;
}

pub trait CodecExt: Codec {
    /// Given a dag, serialize it to bytes.
    fn encode<S>(dag: &S) -> Result<Box<[u8]>, Self::Error>
    where
        S: Serialize;

    /// Given some bytes, deserialize it to a dag.
    fn decode<'de, D>(bytes: &'de [u8]) -> Result<D, Self::Error>
    where
        D: Deserialize<'de>;

    ///
    /// Because some codecs are text-based rather than binary, `Codec`s may define
    /// custom default behaviour for serializing bytes.
    fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    /// Serialize an IPLD link.
    ///
    /// Default behaviour is to serialize the link directly as bytes.
    fn serialize_link<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(cid.to_bytes().as_ref())
    }

    /// Deserialize an unknown Serde type.
    ///
    /// Because the IPLD data model doesn't map 1:1 with the Serde data model,
    /// a type's `Visitor` may be asked to visit an enum or a newtype struct.
    /// In these cases, the type can hand off
    fn deserialize_unknown<'de, D, V>(deserializer: D, visitor: V) -> Result<V::Value, D::Error>
    where
        D: Deserializer<'de>,
        V: DecodeVisitor<'de>,
    {
        deserializer.deserialize_bytes(visitor)
    }
}

/// Allows any implementor to dictate how it maps itself (as one of the IPLD data model
/// types) to `serde`'s data model.
///
/// NOTE: the implementor must also
pub trait Encode: Serialize {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

/// Allows any implementor to
pub trait Decode<'de>: Deserialize<'de> {
    fn decode<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// A helper trait for visiting a link, used by types that need `Cid`s.
pub trait DecodeVisitor<'de>: Visitor<'de> {
    fn visit_link<E>(self, cid: Cid) -> Result<<Self as Visitor<'de>>::Value, E>
    where
        E: de::Error;
}

/// Derives `Serialize` and `Deserialize` for a type, delegating to the underlying
/// `Encode` and `Decode` implementation.
#[macro_export]
macro_rules! serde_to_codec {
    ($encode_type:ty, $decode_type:ty) => {
        impl<'a, C> Serialize for $encode_type
        where
            C: CodecExt,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.encode(serializer)
            }
        }

        impl<'de, C> Deserialize<'de> for $decode_type
        where
            C: CodecExt,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::decode(deserializer)
            }
        }
    }
}
