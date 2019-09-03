//! `Ipld` codecs.
use crate::error::Result;
use crate::ipld::Ipld;

pub mod cbor;
pub mod json;
pub mod pb;

pub use self::cbor::DagCbor;
pub use self::json::DagJson;
pub use self::pb::DagProtobuf;

/// Codec trait.
pub trait IpldCodec {
    /// Data type.
    type Data;
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Encode function.
    fn encode(ipld: &Ipld) -> Result<Self::Data>;
    /// Decode function.
    fn decode(data: &Self::Data) -> Result<Ipld>;
}

/// Binary trait.
pub trait ToBytes: IpldCodec {
    /// Converts `Ipld` to bytes.
    fn to_bytes(ipld: &Ipld) -> Result<Box<[u8]>>;
    /// Parses `Ipld` from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Ipld>;
}

/// String trait.
pub trait ToString: IpldCodec {
    /// Converts `Ipld` to string.
    fn to_string(ipld: &Ipld) -> Result<String>;
    /// Parses `Ipld` from a string slice.
    fn from_str(string: &str) -> Result<Ipld>;
}

impl<T: ToString> ToBytes for T {
    fn to_bytes(ipld: &Ipld) -> Result<Box<[u8]>> {
        Ok(Self::to_string(ipld)?.into_bytes().into_boxed_slice())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ipld> {
        Self::from_str(std::str::from_utf8(bytes)?)
    }
}

/// Block codec.
pub trait Codec: IpldCodec + ToBytes {}

impl<T: IpldCodec + ToBytes> Codec for T {}
