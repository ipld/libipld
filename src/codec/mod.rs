//! `Ipld` codecs.
use crate::Ipld;

pub mod cbor;
pub mod json;
pub mod pb;

pub use self::cbor::DagCbor;
pub use self::json::DagJson;
pub use self::pb::DagProtobuf;

/// Codec trait.
pub trait Codec {
    /// Data type.
    type Data;
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Encode function.
    fn encode(ipld: &Ipld) -> Self::Data;
    /// Decode function.
    fn decode(data: &Self::Data) -> Ipld;
}

/// Binary trait.
pub trait ToBytes: Codec {
    /// Error type.
    type Error;
    /// Converts `Ipld` to bytes.
    fn to_bytes(ipld: &Ipld) -> Vec<u8>;
    /// Parses `Ipld` from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Ipld, Self::Error>;
}

/// String trait.
pub trait ToString: Codec {
    /// Error type.
    type Error;
    /// Converts `Ipld` to string.
    fn to_string(ipld: &Ipld) -> String;
    /// Parses `Ipld` from a string slice.
    fn from_str(string: &str) -> Result<Ipld, Self::Error>;
}
