//! Json codec.
use libipld_base::cid;
use libipld_base::codec::Codec;
use libipld_base::error::BlockError;
use libipld_base::ipld::Ipld;

mod codec;

/// Json codec.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DagJsonCodec;

impl Codec for DagJsonCodec {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagJSON;

    type Error = BlockError;

    fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error> {
        codec::encode(ipld).map_err(|e| BlockError::CodecError(e.into()))
    }

    fn decode(data: &[u8]) -> Result<Ipld, Self::Error> {
        codec::decode(data).map_err(|e| BlockError::CodecError(e.into()))
    }
}
