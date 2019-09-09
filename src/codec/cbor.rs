//! CBOR codec.
use crate::codec::Codec;
use crate::error::Result;
use crate::ipld::{Ipld, IpldRef};

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

/*fn encode(ipld: &Ipld) -> Result<Value> {
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
}*/

impl Codec for DagCbor {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode<'a>(_ipld: IpldRef<'a>) -> Result<Box<[u8]>> {
        Ok(Default::default())
    }

    fn decode(_data: &[u8]) -> Result<Ipld> {
        Ok(Ipld::Null)
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
        let ipld2 = DagCbor::decode(&DagCbor::encode(ipld.as_ref()).unwrap()).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
