//! Protobuf codec.
pub use crate::codec::{PbLink, PbNode};
use core::convert::TryInto;
use libipld_core::codec::{Code, Codec, Decode, Encode};
use libipld_core::ipld::Ipld;
use std::io::{Read, Write};
use thiserror::Error;

mod codec;

/// Protobuf codec.
#[derive(Clone, Copy, Debug)]
pub struct DagPbCodec;

impl Codec for DagPbCodec {
    const CODE: Code = Code::DagProtobuf;

    type Error = Error;
}

/// Protobuf error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Prost(#[from] prost::DecodeError),
    #[error("{0}")]
    Cid(#[from] libipld_core::cid::Error),
    #[error("{0}")]
    TypeError(#[from] libipld_core::error::TypeError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Encode<DagPbCodec> for Ipld {
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        let pb_node: PbNode = self.try_into()?;
        let bytes = pb_node.into_bytes();
        w.write_all(&bytes)?;
        Ok(())
    }
}

impl Decode<DagPbCodec> for Ipld {
    fn decode<R: Read>(r: &mut R) -> Result<Self, Error> {
        let mut bytes = Vec::new();
        r.read_to_end(&mut bytes)?;
        Ok(PbNode::from_bytes(&bytes)?.into())
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
