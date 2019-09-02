//! Protobuf codec.
use super::*;
use crate::untyped::Ipld;

mod dag_pb;
mod gen;

/// Protobuf codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagProtobuf;

impl Codec for DagProtobuf {
    type Data = dag_pb::PbNode;

    const VERSION: cid::Version = cid::Version::V0;
    const CODEC: cid::Codec = cid::Codec::DagProtobuf;

    fn encode(ipld: &Ipld) -> Self::Data {
        dag_pb::PbNode::from(ipld)
    }

    fn decode(data: &Self::Data) -> Ipld {
        data.to_owned().into()
    }
}

impl ToBytes for DagProtobuf {
    type Error = failure::Error;

    fn to_bytes(ipld: &Ipld) -> Vec<u8> {
        let data = Self::encode(ipld);
        data.into_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ipld, Self::Error> {
        let data = dag_pb::PbNode::from_bytes(bytes)?;
        Ok(Self::decode(&data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ipld, pb_cid};

    #[test]
    fn test_encode_decode() {
        let link = ipld!(null);
        let ipld = ipld!({
            "Links": [{
                "Hash": pb_cid!(link),
                "Name": "hello",
                "Tsize": 13u64,
            }],
            "Data": vec![0, 1, 2, 3],
        });
        let ipld2 = DagProtobuf::decode(&DagProtobuf::encode(&ipld));
        assert_eq!(ipld, ipld2);
    }
}
