//! `Ipld` codecs.
use crate::error::Result;
use crate::ipld::Ipld;

pub mod cbor;

pub use self::cbor::DagCborCodec;

/// Codec trait.
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Encode function.
    fn encode(ipld: &Ipld) -> Result<Box<[u8]>>;
    /// Decode function.
    fn decode(data: &[u8]) -> Result<Ipld>;
}
