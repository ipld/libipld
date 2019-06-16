//! CBOR codec.
use super::*;
use crate::ipld::*;
use crate::untyped::Ipld;
use serde_cbor::{ObjectKey, Value};

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

fn encode(ipld: &Ipld) -> Value {
    match ipld {
        Ipld::Null(IpldNull) => Value::Null,
        Ipld::Bool(IpldBool(b)) => Value::Bool(*b),
        Ipld::Integer(IpldInteger::U64(i)) => Value::U64(*i),
        Ipld::Integer(IpldInteger::I64(i)) => Value::I64(*i),
        Ipld::Float(IpldFloat(float)) => Value::F64(*float),
        Ipld::Bytes(IpldBytes(bytes)) => Value::Bytes(bytes.to_owned()),
        Ipld::String(IpldString(string)) => Value::String(string.to_owned()),
        Ipld::List(IpldList(list)) => Value::Array(list.iter().map(encode).collect()),
        Ipld::Map(IpldMap(map)) => {
            let cbor_map = map.iter()
                .map(|(k, v)| (ObjectKey::String(k.to_owned()), encode(v)))
                .collect();
            Value::Object(cbor_map)
        }
        Ipld::Link(IpldLink(cid)) => {
            // TODO tag 42
            Value::Bytes(cid.to_bytes())
        }
    }
}

fn decode(cbor: &Value) -> Ipld {
    match cbor {
        Value::Null => Ipld::Null(IpldNull),
        Value::Bool(b) => Ipld::Bool(IpldBool(*b)),
        Value::U64(i) => Ipld::Integer(IpldInteger::U64(*i)),
        Value::I64(i) => Ipld::Integer(IpldInteger::I64(*i)),
        Value::F64(f) => Ipld::Float(IpldFloat(*f)),
        Value::Bytes(bytes) => Ipld::Bytes(IpldBytes(bytes.to_owned())),
        Value::String(string) => Ipld::String(IpldString(string.to_owned())),
        Value::Array(array) => Ipld::List(IpldList(array.iter().map(decode).collect())),
        Value::Object(object) => {
            let map = object.iter()
                .map(|(k, v)| (k.as_string().unwrap().to_owned(), decode(v)))
                .collect();
            Ipld::Map(IpldMap(map))
        }
    }
}

impl Codec for DagCbor {
    type Data = serde_cbor::Value;

    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode(ipld: &Ipld) -> Self::Data {
        encode(ipld)
    }

    fn decode(data: &Self::Data) -> Ipld {
        decode(data)
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

#[cfg(test)]
mod tests {
    use super::*;
    //use crate::cbor_cid;
    use crate::ipld;

    #[test]
    fn encode_decode_cbor() {
        //let link = ipld!(null);
        let ipld = ipld!({
          "number": 1,
          "list": [true, null],
          "bytes": vec![0, 1, 2, 3],
          //"link": cbor_cid!(link),
        });
        let ipld2 = DagCbor::decode(&DagCbor::encode(&ipld));
        assert_eq!(ipld, ipld2);
    }
}
