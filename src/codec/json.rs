//! JSON codec.
use super::*;
use cid::Cid;
use core::convert::TryFrom;
use crate::ipld::Ipld;
use multibase::Base;
use serde_json::{json, Number, Value};

/// JSON codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagJson;

fn encode(ipld: &Ipld) -> Value {
    match ipld {
        Ipld::Null => Value::Null,
        Ipld::Bool(b) => Value::Bool(*b),
        Ipld::Integer(i) => {
            if *i >= 0 {
                Value::Number(Number::from(*i as u64))
            } else {
                Value::Number(Number::from(*i as i64))
            }
        }
        Ipld::Float(f) => {
            let num = Number::from_f64(*f).expect("not NaN");
            Value::Number(num)
        }
        Ipld::Bytes(b) => {
            let alphabet = Base::Base64.alphabet();
            json!({
                "/": { "base64": base_x::encode(alphabet, b) }
            })
        }
        Ipld::String(s) => Value::String(s.to_owned()),
        Ipld::List(l) => Value::Array(l.iter().map(encode).collect()),
        Ipld::Map(m) => {
            Value::Object(m.iter().map(|(k, v)| (k.to_owned(), encode(v))).collect())
        }
        Ipld::Link(cid) => json!({
            "/": cid.to_string()
        }),
    }
}

fn decode(json: &Value) -> Ipld {
    match json {
        Value::Null => Ipld::Null,
        Value::Bool(b) => Ipld::Bool(*b),
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                return Ipld::Integer(i as i128);
            }
            if let Some(i) = num.as_u64() {
                return Ipld::Integer(i as i128);
            }
            if let Some(f) = num.as_f64() {
                return Ipld::Float(f);
            }
            Ipld::Null
        }
        Value::String(s) => Ipld::String(s.to_owned()),
        Value::Array(array) => Ipld::List(array.iter().map(decode).collect()),
        Value::Object(object) => {
            match object.get("/") {
                Some(Value::String(string)) => {
                    if let Some(cid) = Cid::try_from(string.as_str()).ok() {
                        return Ipld::Link(cid);
                    }
                }
                Some(Value::Object(object)) => {
                    if let Value::String(string) = &object["base64"] {
                        let alphabet = Base::Base64.alphabet();
                        if let Some(bytes) = base_x::decode(alphabet, &string).ok() {
                            return Ipld::Bytes(bytes);
                        }
                    }
                }
                _ => {}
            }
            let map = object.iter().map(|(k, v)| (k.to_owned(), decode(v))).collect();
            Ipld::Map(map)
        }
    }
}

impl Codec for DagJson {
    type Data = serde_json::Value;

    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagJSON;

    fn encode(ipld: &Ipld) -> Self::Data {
        encode(ipld)
    }

    fn decode(data: &Self::Data) -> Ipld {
        decode(data)
    }
}

impl ToString for DagJson {
    type Error = serde_json::Error;

    fn to_string(ipld: &Ipld) -> String {
        let data = Self::encode(ipld);
        serde_json::to_string(&data).expect("cannot fail")
    }

    fn from_str(string: &str) -> Result<Ipld, Self::Error> {
        let data = serde_json::from_str(string)?;
        Ok(Self::decode(&data))
    }
}

impl ToBytes for DagJson {
    type Error = serde_json::Error;

    fn to_bytes(ipld: &Ipld) -> Vec<u8> {
        Self::to_string(ipld).into_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Ipld, Self::Error> {
        Self::from_str(std::str::from_utf8(bytes).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld;
    use crate::json_cid;
    use serde_json::json;

    #[test]
    fn encode_json() {
        let link = ipld!(null);
        let ipld = ipld!({
            "number": 1,
            "list": [true, null],
            "bytes": vec![0, 1, 2, 3],
            "link": json_cid!(link),
        });
        let json = json!({
            "number": 1,
            "list": [true, null],
            "bytes": {
                "/": { "base64": "AQID" },
            },
            "link": {
                "/": json_cid!(link).to_string(),
            }
        });
        let json2 = DagJson::encode(&ipld);
        assert_eq!(json, json2);
    }

    #[test]
    fn decode_json() {
        let link = ipld!(null);
        let ipld = ipld!({
            "number": 1,
            "list": [true, null],
            "bytes": vec![0, 1, 2, 3],
            "link": json_cid!(link),
        });
        let json = json!({
            "number": 1,
            "list": [true, null],
            "bytes": {
                "/": { "base64": "AQID" },
            },
            "link": {
                "/": json_cid!(link).to_string(),
            }
        });
        let ipld2 = DagJson::decode(&json);
        assert_eq!(ipld, ipld2);
    }
}
