//! Json codec.
use libipld_base::cid;
use libipld_base::codec::Codec;
use libipld_base::error::{BlockError, IpldError};
use libipld_base::ipld::Ipld;
use thiserror::Error;

mod codec;

/// Json codec.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DagJsonCodec;

impl Codec for DagJsonCodec {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagJSON;

    type Error = JsonError;

    fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error> {
        codec::encode(ipld)
    }

    fn decode(data: &[u8]) -> Result<Ipld, Self::Error> {
        codec::decode(data)
    }
}

/// Json error.
#[derive(Debug, Error)]
pub enum JsonError {
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Cid(#[from] cid::Error),
    #[error("{0}")]
    Ipld(#[from] IpldError),
}

impl From<JsonError> for BlockError {
    fn from(error: JsonError) -> Self {
        Self::CodecError(error.into())
    }
}
