//! `Ipld` codecs.
use crate::error::BlockError;
use crate::ipld::Ipld;
use async_std::io::{Read, Seek, Write};
use async_trait::async_trait;
use core::fmt::Debug;
use failure::Fail;

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
