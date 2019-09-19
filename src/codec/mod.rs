//! `Ipld` codecs.
use crate::error::Result;
use crate::ipld::Ipld;
use async_trait::async_trait;

pub mod cbor;

pub use self::cbor::DagCborCodec;

/// Codec trait.
#[async_trait]
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;
    /// Codec code.
    const CODEC: cid::Codec;
    /// Encode function.
    async fn encode(ipld: &Ipld) -> Result<Box<[u8]>>;
    /// Decode function.
    async fn decode(data: &[u8]) -> Result<Ipld>;
}
