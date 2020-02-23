//! `Ipld` codecs.
use crate::error::BlockError;
use crate::ipld::Ipld;
use core::fmt::Debug;
use failure::Fail;

/// Codec trait.
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Error type.
    type Error: Debug + Fail + Into<BlockError>;
    /// Encode function.
    fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error>;
    /// Decode function.
    fn decode(data: &[u8]) -> Result<Ipld, Self::Error>;
}
