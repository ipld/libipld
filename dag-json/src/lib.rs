//! Json codec.
use libipld_core::cid;
use libipld_core::codec::Codec;
use libipld_core::error::BlockError;
use libipld_core::ipld::Ipld;

mod codec;

/// Json codec.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DagJsonCodec;

impl DagJsonCodec {
    pub const CODEC: cid::Codec = cid::Codec::DagJSON;

    pub fn encode(ipld: &Ipld) -> Result<Box<[u8]>, BlockError> {
        codec::encode(ipld).map_err(|e| BlockError::CodecError(e.into()))
    }

    pub fn decode(data: &[u8]) -> Result<Ipld, BlockError> {
        codec::decode(data).map_err(|e| BlockError::CodecError(e.into()))
    }
}

impl Codec for DagJsonCodec {
    fn codec(&self) -> cid::Codec {
        Self::CODEC
    }
    fn encode(&self, ipld: &Ipld) -> Result<Box<[u8]>, BlockError> {
        Self::encode(ipld).map_err(|err| err.into())
    }

    fn decode(&self, data: &[u8]) -> Result<Ipld, BlockError> {
        Self::decode(data).map_err(|err| err.into())
    }
}
