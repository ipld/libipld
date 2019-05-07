//! CBOR codec.
use super::*;
use crate::ipld::IpldNull;
use crate::untyped::Ipld;

/// CBOR codec.
pub struct DagCbor;

impl Codec for DagCbor {
    type Data = serde_cbor::Value;

    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode(_ipld: &Ipld) -> Self::Data {
        serde_cbor::Value::Null
    }

    fn decode(_data: &Self::Data) -> Ipld {
        Ipld::Null(IpldNull)
    }
}

impl ToBytes for DagCbor {
    type Error = serde_cbor::error::Error;

    fn to_bytes(ipld: &Ipld) -> Vec<u8> {
        let data = Self::encode(ipld);
        serde_cbor::to_vec(&data).expect("cannot fail")
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ipld, Self::Error> {
        let data = serde_cbor::from_slice(bytes)?;
        Ok(Self::decode(&data))
    }
}
