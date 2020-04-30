//! `Ipld` codecs.
use crate::error::BlockError;
use crate::ipld::Ipld;
use std::error::Error;

/// Codec trait.
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Error type.
    type Error: Error + Into<BlockError>;
    /// Encode function.
    fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error>;
    /// Decode function.
    fn decode(data: &[u8]) -> Result<Ipld, Self::Error>;
}
