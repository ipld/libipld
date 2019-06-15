//! JSON codec.
use super::*;
use crate::ipld::*;
use crate::untyped::Ipld;
use multibase::Base;
use serde_json::{json, Number, Value};

/// JSON codec.
pub struct DagJson;

fn encode(ipld: &Ipld) -> Value {
    match ipld {
        Ipld::Null(IpldNull) => Value::Null,
        Ipld::Bool(IpldBool(b)) => Value::Bool(*b),
        Ipld::Integer(IpldInteger::U64(i)) => Value::Number(Number::from(*i)),
        Ipld::Integer(IpldInteger::I64(i)) => Value::Number(Number::from(*i)),
        Ipld::Float(IpldFloat(float)) => {
            let num = Number::from_f64(*float).expect("not NaN");
            Value::Number(num)
        }
        Ipld::Bytes(IpldBytes(bytes)) => {
            let alphabet = Base::Base64.alphabet();
            json!({
                "/": { "base64": base_x::encode(alphabet, bytes) }
            })
        }
        Ipld::String(IpldString(string)) => Value::String(string.to_owned()),
        Ipld::List(IpldList(list)) => Value::Array(list.iter().map(encode).collect()),
        Ipld::Map(IpldMap(map)) => {
            Value::Object(map.iter().map(|(k, v)| (k.to_owned(), encode(v))).collect())
        }
        Ipld::Link(IpldLink(cid)) => json!({
            "/": cid.to_string()
        }),
    }
}
impl Codec for DagJson {
    type Data = serde_json::Value;

    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagJSON;

    fn encode(ipld: &Ipld) -> Self::Data {
        encode(ipld)
    }

    fn decode(_data: &Self::Data) -> Ipld {
        Ipld::Null(IpldNull)
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
}
