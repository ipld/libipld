//! Protobuf codec.
use super::*;
use crate::error::Result;
use crate::ipld::Ipld;
use core::convert::TryFrom;

mod dag_pb;
mod gen;

/// Protobuf codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagProtobuf;

impl IpldCodec for DagProtobuf {
    type Data = dag_pb::PbNode;

    const VERSION: cid::Version = cid::Version::V0;
    const CODEC: cid::Codec = cid::Codec::DagProtobuf;

    fn encode(ipld: &Ipld) -> Result<Self::Data> {
        dag_pb::PbNode::try_from(ipld)
    }

    fn decode(data: &Self::Data) -> Result<Ipld> {
        Ok(data.to_owned().into())
    }
}

impl ToBytes for DagProtobuf {
    fn to_bytes(ipld: &Ipld) -> Result<Box<[u8]>> {
        let data = Self::encode(ipld)?;
        Ok(data.into_bytes()?.into_boxed_slice())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ipld> {
        let data = dag_pb::PbNode::from_bytes(bytes)?;
        Self::decode(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block, ipld};

    #[test]
    fn test_encode_decode() {
        let link = block!({}).unwrap();
        let ipld = ipld!({
            "Links": [{
                "Hash": link.cid(),
                "Name": "hello",
                "Tsize": 13u64,
            }],
            "Data": vec![0, 1, 2, 3],
        });
        let ipld2 = DagProtobuf::decode(&DagProtobuf::encode(&ipld).unwrap()).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
