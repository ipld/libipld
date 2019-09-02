//! CBOR codec.
use super::*;
use crate::ipld::Ipld;
use serde_cbor::Value;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

fn encode(ipld: &Ipld) -> Value {
    match ipld {
        Ipld::Null => Value::Null,
        Ipld::Bool(b) => Value::Bool(*b),
        Ipld::Integer(i) => Value::Integer(*i),
        Ipld::Float(f) => Value::Float(*f),
        Ipld::Bytes(b) => Value::Bytes(b.to_owned()),
        Ipld::String(s) => Value::Text(s.to_owned()),
        Ipld::List(l) => Value::Array(l.iter().map(encode).collect()),
        Ipld::Map(m) => {
            let cbor_map = m.iter()
                .map(|(k, v)| (Value::Text(k.to_owned()), encode(v)))
                .collect();
            Value::Map(cbor_map)
        }
        Ipld::Link(cid) => {
            // TODO tag 42
            Value::Bytes(cid.to_bytes())
        }
    }
}

fn decode(cbor: &Value) -> Ipld {
    match cbor {
        Value::Null => Ipld::Null,
        Value::Bool(b) => Ipld::Bool(*b),
        Value::Integer(i) => Ipld::Integer(*i),
        Value::Float(f) => Ipld::Float(*f),
        Value::Bytes(bytes) => Ipld::Bytes(bytes.to_owned()),
        Value::Text(string) => Ipld::String(string.to_owned()),
        Value::Array(array) => Ipld::List(array.iter().map(decode).collect()),
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
            Ipld::Map(map)
        },
        Value::__Hidden => Ipld::Null,
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
