use core::convert::TryFrom;
use libipld_core::cid::CidGeneric;
use libipld_core::ipld::Ipld;
use serde::de::Error as SerdeError;
use serde::{de, ser, Deserialize, Serialize};
use serde_json::ser::Serializer;
use serde_json::Error;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::marker::PhantomData;

const LINK_KEY: &str = "/";

pub fn encode<W, H>(ipld: &Ipld<H>, writer: &mut W) -> Result<(), Error>
where
    W: Write,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    let mut ser = Serializer::new(writer);
    serialize(&ipld, &mut ser)?;
    Ok(())
}

pub fn decode<R, H>(r: &mut R) -> Result<Ipld<H>, Error>
where
    R: Read,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    let mut de = serde_json::Deserializer::from_reader(r);
    Ok(deserialize(&mut de)?)
}

fn serialize<S, H>(ipld: &Ipld<H>, ser: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    match &ipld {
        Ipld::Null => ser.serialize_none(),
        Ipld::Bool(bool) => ser.serialize_bool(*bool),
        Ipld::Integer(i128) => ser.serialize_i128(*i128),
        Ipld::Float(f64) => ser.serialize_f64(*f64),
        Ipld::String(string) => ser.serialize_str(&string),
        Ipld::Bytes(bytes) => ser.serialize_bytes(&bytes),
        Ipld::List(list) => {
            let wrapped = list.iter().map(|ipld| Wrapper(ipld));
            ser.collect_seq(wrapped)
        }
        Ipld::Map(map) => {
            let wrapped = map.iter().map(|(key, ipld)| (key, Wrapper(ipld)));
            ser.collect_map(wrapped)
        }
        Ipld::Link(link) => {
            let value = base64::encode(&link.to_bytes());
            let mut map = BTreeMap::new();
            map.insert("/", value);

            ser.collect_map(map)
        }
    }
}

fn deserialize<'de, D, H>(deserializer: D) -> Result<Ipld<H>, D::Error>
where
    D: de::Deserializer<'de>,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    // Sadly such a PhantomData hack is needed
    deserializer.deserialize_any(JSONVisitor(PhantomData))
}

// Needed for `collect_seq` and `collect_map` in Seserializer
struct Wrapper<'a, H>(&'a Ipld<H>)
where
    H: Into<u64> + TryFrom<u64> + Copy;
impl<'a, H> Serialize for Wrapper<'a, H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serialize(&self.0, serializer)
    }
}

// serde deserializer visitor that is used by Deseraliazer to decode
// json into IPLD.
struct JSONVisitor<H>(PhantomData<H>);
impl<'de, H> de::Visitor<'de> for JSONVisitor<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    type Value = Ipld<H>;

    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("any valid JSON value")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(String::from(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::String(value))
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_byte_buf(v.to_owned())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Bytes(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v.into()))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v.into()))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Bool(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_unit()
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Null)
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut vec: Vec<WrapperOwned<H>> = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }

        let unwrapped = vec.into_iter().map(|WrapperOwned(ipld)| ipld).collect();
        Ok(Ipld::List(unwrapped))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut values: Vec<(String, WrapperOwned<H>)> = Vec::new();

        while let Some((key, value)) = visitor.next_entry()? {
            values.push((key, value));
        }

        // JSON Object represents IPLD Link if it is `{ "/": "...." }` therefor
        // we valiadet if that is the case here.
        if let Some((key, WrapperOwned(Ipld::String(value)))) = values.first() {
            if key == LINK_KEY && values.len() == 1 {
                let link = base64::decode(value).map_err(SerdeError::custom)?;
                let cid = CidGeneric::<u64, H>::try_from(link).map_err(SerdeError::custom)?;
                return Ok(Ipld::Link(cid));
            }
        }

        let unwrapped = values
            .into_iter()
            .map(|(key, WrapperOwned(value))| (key, value));
        Ok(Ipld::Map(BTreeMap::from_iter(unwrapped)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Float(v))
    }
}

// Needed for `visit_seq` and `visit_map` in Deserializer
/// We cannot directly implement `serde::Deserializer` for `Ipld` as it is a remote type.
/// Instead wrap it into a newtype struct and implement `serde::Deserialize` for that one.
/// All the deserializer does is calling the `deserialize()` function we defined which returns
/// an unwrapped `Ipld` instance. Wrap that `Ipld` instance in `Wrapper` and return it.
/// Users of this wrapper will then unwrap it again so that they can return the expected `Ipld`
/// instance.
struct WrapperOwned<H>(Ipld<H>)
where
    H: Into<u64> + TryFrom<u64> + Copy;
impl<'de, H> Deserialize<'de> for WrapperOwned<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let deserialized = deserialize(deserializer);
        // Better version of Ok(Wrapper(deserialized.unwrap()))
        deserialized.map(Self)
    }
}
