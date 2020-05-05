//! Protobuf codec.
pub use crate::codec::{PbLink, PbNode};
use core::convert::TryInto;
use libipld_core::cid;
use libipld_core::codec::Codec;
use libipld_core::error::{BlockError, IpldError};
use libipld_core::ipld::Ipld;
use thiserror::Error;

mod codec;

/// Protobuf codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagPbCodec;

impl DagPbCodec {
    pub const CODEC: cid::Codec = cid::Codec::DagProtobuf;

    pub fn encode(ipld: &Ipld) -> Result<Box<[u8]>, ProtobufError> {
        let pb_node: PbNode = ipld.try_into()?;
        Ok(pb_node.into_bytes())
    }

    pub fn decode(data: &[u8]) -> Result<Ipld, ProtobufError> {
        Ok(PbNode::from_bytes(data)?.into())
    }
}

impl Codec for DagPbCodec {
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

/// Protobuf error.
#[derive(Debug, Error)]
pub enum ProtobufError {
    #[error("{0}")]
    Prost(#[from] prost::DecodeError),
    #[error("{0}")]
    Cid(#[from] cid::Error),
    #[error("{0}")]
    Ipld(#[from] IpldError),
}

impl From<ProtobufError> for BlockError {
    fn from(error: ProtobufError) -> Self {
        Self::CodecError(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::multihash::Sha2_256;
    use std::collections::BTreeMap;

    #[test]
    fn test_encode_decode() {
        let cid = Cid::new_v0(Sha2_256::digest(b"cid")).unwrap();
        let mut pb_link = BTreeMap::<String, Ipld>::new();
        pb_link.insert("Hash".to_string(), cid.into());
        pb_link.insert("Name".to_string(), "block".to_string().into());
        pb_link.insert("Tsize".to_string(), 13.into());

        let links: Vec<Ipld> = vec![pb_link.into()];
        let mut pb_node = BTreeMap::<String, Ipld>::new();
        pb_node.insert("Data".to_string(), b"Here is some data\n".to_vec().into());
        pb_node.insert("Links".to_string(), links.into());
        let data: Ipld = pb_node.into();

        let bytes = DagPbCodec::encode(&data).unwrap();
        let data2 = DagPbCodec::decode(&bytes).unwrap();
        assert_eq!(data, data2);
    }
}
