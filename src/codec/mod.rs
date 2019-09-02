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
pub trait Codec {
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
pub trait ToBytes: Codec {
    /// Converts `Ipld` to bytes.
    fn to_bytes(ipld: &Ipld) -> Result<Vec<u8>>;
    /// Parses `Ipld` from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Ipld>;
}

/// String trait.
pub trait ToString: Codec {
    /// Converts `Ipld` to string.
    fn to_string(ipld: &Ipld) -> Result<String>;
    /// Parses `Ipld` from a string slice.
    fn from_str(string: &str) -> Result<Ipld>;
}
