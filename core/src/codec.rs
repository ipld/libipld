//! `Ipld` codecs.
use crate::error::BlockError;
use crate::ipld::Ipld;

/// Codec trait.
///
/// This trait is used for trait objects.
pub trait Codec {
    /// Codec code.
    fn codec(&self) -> cid::Codec;
    /// Encode function.
    fn encode(&self, ipld: &Ipld) -> Result<Box<[u8]>, BlockError>;
    /// Decode function.
    fn decode(&self, data: &[u8]) -> Result<Ipld, BlockError>;
}
