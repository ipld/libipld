//! `Ipld` codecs.
use crate::error::{format_err, Result};
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

/// Decode bytes.
pub fn decode(codec: cid::Codec, data: &[u8]) -> Result<Ipld> {
    match codec {
        cid::Codec::DagCBOR => DagCborCodec::decode(&data),
        _ => Err(format_err!("unsupported codec {:?}", codec)),
    }
}
