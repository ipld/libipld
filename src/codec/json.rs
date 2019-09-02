//! JSON codec.
use super::*;
use crate::error::{format_err, Result};
use crate::ipld::Ipld;
use cid::Cid;
use core::convert::TryFrom;
use multibase::Base;
use serde_json::{json, Number, Value};
use std::collections::HashMap;

/// JSON codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagJson;

fn encode(ipld: &Ipld) -> Result<Value> {
    let json = match ipld {
        Ipld::Null => Value::Null,
        Ipld::Bool(b) => Value::Bool(*b),
        Ipld::Integer(int) => {
            let num = if *int < 0 {
                let i: i64 = TryFrom::try_from(*int)?;
                Number::from(i)
            } else {
                let u: u64 = TryFrom::try_from(*int)?;
                Number::from(u)
            };
            Value::Number(num)
        }
        Ipld::Float(f) => {
            let num = if let Some(num) = Number::from_f64(*f) {
                num
            } else {
                return Err(format_err!("float is NaN"));
            };
            Value::Number(num)
        }
        Ipld::Bytes(b) => {
            let alphabet = Base::Base64.alphabet();
            json!({
                "/": { "base64": base_x::encode(alphabet, b) }
            })
        }
        Ipld::String(s) => Value::String(s.to_owned()),
        Ipld::List(list) => {
            let mut array = Vec::with_capacity(list.len());
            for item in list.iter() {
                array.push(encode(item)?);
            }
            Value::Array(array)
        }
        Ipld::Map(map) => {
            let object = map
                .iter()
                .map(|(k, v)| Ok((k.to_owned(), encode(v)?)))
                .collect::<Result<_>>()?;
            Value::Object(object)
        }
        Ipld::Link(cid) => json!({
            "/": cid.to_string()
        }),
    };
    Ok(json)
}

fn decode(json: &Value) -> Result<Ipld> {
    let ipld = match json {
        Value::Null => Ipld::Null,
        Value::Bool(b) => Ipld::Bool(*b),
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                return Ok(Ipld::Integer(i.into()));
            }
            if let Some(i) = num.as_u64() {
                return Ok(Ipld::Integer(i.into()));
            }
            if let Some(f) = num.as_f64() {
                return Ok(Ipld::Float(f));
            }
            return Err(format_err!("invalid number"));
        }
        Value::String(s) => Ipld::String(s.to_owned()),
        Value::Array(array) => {
            let mut list = Vec::with_capacity(array.len());
            for item in array.iter() {
                list.push(decode(item)?);
            }
            Ipld::List(list)
        }
        Value::Object(object) => match object.get("/") {
            Some(Value::String(string)) => Ipld::Link(Cid::try_from(string.as_str())?),
            Some(Value::Object(object)) => {
                if let Value::String(string) = &object["base64"] {
                    let alphabet = Base::Base64.alphabet();
                    Ipld::Bytes(base_x::decode(alphabet, &string)?)
                } else {
                    return Err(format_err!("expected base64 key"));
                }
            }
            None => {
                let mut map = HashMap::with_capacity(object.len());
                for (k, v) in object.iter() {
                    map.insert(k.to_owned(), decode(v)?);
                }
                Ipld::Map(map)
            }
            _ => return Err(format_err!("expected bytes or cid")),
        },
    };
    Ok(ipld)
}

impl IpldCodec for DagJson {
    type Data = serde_json::Value;

    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagJSON;

    fn encode(ipld: &Ipld) -> Result<Self::Data> {
        encode(ipld)
    }

    fn decode(data: &Self::Data) -> Result<Ipld> {
        decode(data)
    }
}

impl ToString for DagJson {
    fn to_string(ipld: &Ipld) -> Result<String> {
        let data = Self::encode(ipld)?;
        Ok(serde_json::to_string(&data)?)
    }

    fn from_str(string: &str) -> Result<Ipld> {
        let data = serde_json::from_str(string)?;
        Ok(Self::decode(&data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ipld, json_block};
    use serde_json::json;

    #[test]
    fn encode_json() {
        let link = json_block!(null).unwrap();
        let ipld = ipld!({
            "number": 1,
            "list": [true, null],
            "bytes": vec![0, 1, 2, 3],
            "link": link.cid(),
        });
        let json = json!({
            "number": 1,
            "list": [true, null],
            "bytes": {
                "/": { "base64": "AQID" },
            },
            "link": {
                "/": link.cid().to_string(),
            }
        });
        let json2 = DagJson::encode(&ipld).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn decode_json() {
        let link = json_block!(null).unwrap();
        let ipld = ipld!({
            "number": 1,
            "list": [true, null],
            "bytes": vec![0, 1, 2, 3],
            "link": link.cid(),
        });
        let json = json!({
            "number": 1,
            "list": [true, null],
            "bytes": {
                "/": { "base64": "AQID" },
            },
            "link": {
                "/": link.cid().to_string(),
            }
        });
        let ipld2 = DagJson::decode(&json).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
