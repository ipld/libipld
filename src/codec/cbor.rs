//! CBOR codec.
use super::*;
use crate::ipld::*;
use crate::untyped::Ipld;
use serde_cbor::Value;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

fn encode(ipld: &Ipld) -> Value {
    match ipld {
        Ipld::Null(IpldNull) => Value::Null,
        Ipld::Bool(IpldBool(b)) => Value::Bool(*b),
        Ipld::Integer(IpldInteger(i)) => Value::Integer(*i),
        Ipld::Float(IpldFloat(float)) => Value::Float(*float),
        Ipld::Bytes(IpldBytes(bytes)) => Value::Bytes(bytes.to_owned()),
        Ipld::String(IpldString(string)) => Value::Text(string.to_owned()),
        Ipld::List(IpldList(list)) => Value::Array(list.iter().map(encode).collect()),
        Ipld::Map(IpldMap(map)) => {
            let cbor_map = map.iter()
                .map(|(k, v)| (Value::Text(k.to_owned()), encode(v)))
                .collect();
            Value::Map(cbor_map)
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
        Value::Integer(i) => Ipld::Integer(IpldInteger(*i)),
        Value::Float(f) => Ipld::Float(IpldFloat(*f)),
        Value::Bytes(bytes) => Ipld::Bytes(IpldBytes(bytes.to_owned())),
        Value::Text(string) => Ipld::String(IpldString(string.to_owned())),
        Value::Array(array) => Ipld::List(IpldList(array.iter().map(decode).collect())),
        Value::Map(object) => {
            let map = object.iter()
                .map(|(k, v)| {
                    if let Value::Text(string) = k {
                        (string.to_owned(), decode(v))
                    } else {
                        panic!("can only use string keys")
                    }
                })
                .collect();
            Ipld::Map(IpldMap(map))
        },
        Value::__Hidden => Ipld::Null(IpldNull),
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
