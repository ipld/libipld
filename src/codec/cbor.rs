//! CBOR codec.
use super::*;
use crate::error::{format_err, Result};
use crate::ipld::Ipld;
use cid::Cid;
use core::convert::{TryFrom, TryInto};
use serde_cbor::Value;
use std::collections::{BTreeMap, HashMap};

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

fn encode(ipld: &Ipld) -> Result<Value> {
    let cbor = match ipld {
        Ipld::Null => Value::Null,
        Ipld::Bool(b) => Value::Bool(*b),
        Ipld::Integer(i) => Value::Integer(*i),
        Ipld::Float(f) => Value::Float(*f),
        Ipld::Bytes(b) => Value::Bytes(b.to_owned()),
        Ipld::String(s) => Value::Text(s.to_owned()),
        Ipld::List(l) => {
            let mut array = Vec::with_capacity(l.len());
            for item in l.iter() {
                array.push(encode(item)?);
            }
            Value::Array(array)
        }
        Ipld::Map(m) => {
            let mut map = BTreeMap::new();
            for (k, v) in m {
                map.insert(encode(&k.to_owned().into())?, encode(v)?);
            }
            Value::Map(map)
        }
        Ipld::Link(cid) => {
            let mut map = BTreeMap::new();
            map.insert(Value::Integer(42), Value::Bytes(cid.to_bytes()));
            Value::Map(map)
        }
    };
    Ok(cbor)
}

fn decode(cbor: &Value) -> Result<Ipld> {
    let ipld = match cbor {
        Value::Null => Ipld::Null,
        Value::Bool(b) => Ipld::Bool(*b),
        Value::Integer(i) => Ipld::Integer(*i),
        Value::Float(f) => Ipld::Float(*f),
        Value::Bytes(bytes) => Ipld::Bytes(bytes.to_owned()),
        Value::Text(string) => Ipld::String(string.to_owned()),
        Value::Array(array) => {
            let mut list = Vec::with_capacity(array.len());
            for item in array.iter() {
                list.push(decode(item)?);
            }
            Ipld::List(list)
        }
        Value::Map(object) => {
            if let Some(Value::Bytes(bytes)) = object.get(&Value::Integer(42)) {
                Ipld::Link(Cid::try_from(bytes.as_slice())?)
            } else {
                let mut map = HashMap::with_capacity(object.len());
                for (k, v) in object.iter() {
                    map.insert(decode(k)?.try_into()?, decode(v)?);
                }
                Ipld::Map(map)
            }
        }
        Value::__Hidden => return Err(format_err!("__Hidden value not supported")),
    };
    Ok(ipld)
}

impl IpldCodec for DagCbor {
    type Data = serde_cbor::Value;

    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode(ipld: &Ipld) -> Result<Self::Data> {
        encode(ipld)
    }

    fn decode(data: &Self::Data) -> Result<Ipld> {
        decode(data)
    }
}

impl ToBytes for DagCbor {
    fn to_bytes(ipld: &Ipld) -> Result<Box<[u8]>> {
        let data = Self::encode(ipld)?;
        Ok(serde_cbor::to_vec(&data)?.into_boxed_slice())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ipld> {
        let data = serde_cbor::from_slice(bytes)?;
        Ok(Self::decode(&data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block, ipld};

    #[test]
    fn encode_decode_cbor() {
        let link = block!(null).unwrap();
        let ipld = ipld!({
          "number": 1,
          "list": [true, null],
          "bytes": vec![0, 1, 2, 3],
          "link": link.cid(),
        });
        let ipld2 = DagCbor::decode(&DagCbor::encode(&ipld).unwrap()).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
