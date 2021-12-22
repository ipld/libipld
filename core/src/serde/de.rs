use std::collections::BTreeMap;
use std::fmt;

use cid::serde::{CidVisitor, CID_SERDE_NEWTYPE_STRUCT_NAME};
use serde::{de, forward_to_deserialize_any};

use crate::error::SerdeError;
use crate::ipld::Ipld;

/// Deserialize instances of [`crate::ipld::Ipld`].
///
/// # Example
///
/// ```
/// use std::collections::BTreeMap;
///
/// use serde::Deserialize;
/// use libipld_core::ipld::Ipld;
/// use libipld_core::serde::from_ipld;
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u8,
///     hobbies: Vec<String>,
///     is_cool: bool,
/// }
///
/// let ipld = Ipld::Map({
///     BTreeMap::from([
///         ("name".into(), Ipld::String("Hello World!".into())),
///         ("age".into(), Ipld::Integer(52)),
///         (
///             "hobbies".into(),
///             Ipld::List(vec![
///                 Ipld::String("geography".into()),
///                 Ipld::String("programming".into()),
///             ]),
///         ),
///         ("is_cool".into(), Ipld::Bool(true)),
///     ])
/// });
///
/// let person = from_ipld(ipld);
/// assert!(matches!(person, Ok(Person { .. })));
/// ```
// NOTE vmx 2021-12-22: Taking by value is also what `serde_json` does.
pub fn from_ipld<T>(value: Ipld) -> Result<T, SerdeError>
where
    T: serde::de::DeserializeOwned,
{
    T::deserialize(value)
}

impl<'de> de::Deserialize<'de> for Ipld {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(IpldVisitor)
    }
}

struct IpldVisitor;

impl<'de> de::Visitor<'de> for IpldVisitor {
    type Value = Ipld;

    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("any valid CBOR value")
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(String::from(value))
    }

    #[inline]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::String(value))
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_byte_buf(v.to_owned())
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Bytes(v))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v.into()))
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v.into()))
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v))
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Float(v))
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Bool(v))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_unit()
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Null)
    }

    #[inline]
    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }

        Ok(Ipld::List(vec))
    }

    #[inline]
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut values = BTreeMap::new();

        while let Some((key, value)) = visitor.next_entry()? {
            values.insert(key, value);
        }

        Ok(Ipld::Map(values))
    }

    /// Newtype structs are only used to deserialize CIDs.
    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // TODO vmx 2021-12-21: Check if it might be possible to create the CID directly
        // here without using a serializer. Perhaps checking that we are in the correct
        // newtype struct and then calling `Cid::deserialize()` direcrtly.
        deserializer
            .deserialize_newtype_struct(CID_SERDE_NEWTYPE_STRUCT_NAME, CidVisitor)
            .map(Ipld::Link)
    }
}

// TODO vmx 2021-12-22: Go over the code and think about the "unreachable" cases.
impl<'de> de::Deserializer<'de> for Ipld {
    type Error = SerdeError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Null => visitor.visit_unit(),
            Self::Bool(bool) => visitor.visit_bool(bool),
            Self::Integer(i128) => visitor.visit_i128(i128),
            Self::Float(f64) => visitor.visit_f64(f64),
            Self::String(string) => visitor.visit_str(&string),
            Self::Bytes(bytes) => visitor.visit_bytes(&bytes),
            Self::List(list) => visit_seq(list, visitor),
            Self::Map(map) => visit_map(map, visitor),
            Self::Link(link) => visitor.visit_bytes(&link.to_bytes()),
        }
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _variants: &[&str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_f32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_f64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_i8(integer as i8),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }
    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_i16(integer as i16),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }
    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_i32(integer as i32),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }
    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_i64(integer as i64),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }
    fn deserialize_ignored_any<V: de::Visitor<'de>>(
        self,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_map<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unreachable!()
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        name: &str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if name == CID_SERDE_NEWTYPE_STRUCT_NAME {
            match self {
                Ipld::Link(_) => {
                    println!(
                        "vmx: core serde de: deserialize newtype struct: {:#?}",
                        self
                    );
                    self.deserialize_bytes(visitor)
                }
                _ => Err(SerdeError(format!(
                    "Only `Ipld::Link`s can be deserialized to CIDs, input was `{:#?}`",
                    self
                ))),
            }
        } else {
            // TODO vmx 2021-12-22: Check if this is actually true of if there could be some newtype
            // struct case.
            unreachable!(
                "This deserializer must not be called on newtype structs other than one named `{}`",
                CID_SERDE_NEWTYPE_STRUCT_NAME
            )
        }
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unreachable!()
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_u8(integer as u8),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_u16(integer as u16),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_u32(integer as u32),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Integer(integer) => visitor.visit_u64(integer as u64),
            _ => panic!("TODO vmx 2021-12-21: add proper error"),
        }
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        unreachable!()
    }
    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unreachable!()
    }

    forward_to_deserialize_any! {
       bytes byte_buf bool identifier seq string struct str
    }
}

fn visit_map<'de, V>(map: BTreeMap<String, Ipld>, visitor: V) -> Result<V::Value, SerdeError>
where
    V: de::Visitor<'de>,
{
    let mut deserializer = MapDeserializer::new(map);
    visitor.visit_map(&mut deserializer)
}

fn visit_seq<'de, V>(list: Vec<Ipld>, visitor: V) -> Result<V::Value, SerdeError>
where
    V: de::Visitor<'de>,
{
    let mut deserializer = SeqDeserializer::new(list);
    visitor.visit_seq(&mut deserializer)
}

// Heavily based on
// https://github.com/serde-rs/json/blob/95f67a09399d546d9ecadeb747a845a77ff309b2/src/value/de.rs#L601
struct MapDeserializer {
    iter: <BTreeMap<String, Ipld> as IntoIterator>::IntoIter,
    value: Option<Ipld>,
}

impl MapDeserializer {
    fn new(map: BTreeMap<String, Ipld>) -> Self {
        Self {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for MapDeserializer {
    type Error = SerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                // TODO vmx 2021-12-21: I'm not sure if wrapping in `Ipld::String` is the right
                // thing to do here.
                seed.deserialize(Ipld::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(de::Error::custom("value is missing")),
        }
    }
}

// Heavily based on
// https://github.com/serde-rs/json/blob/95f67a09399d546d9ecadeb747a845a77ff309b2/src/value/de.rs#L554
struct SeqDeserializer {
    iter: <Vec<Ipld> as IntoIterator>::IntoIter,
}

impl SeqDeserializer {
    fn new(vec: Vec<Ipld>) -> Self {
        Self {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> de::SeqAccess<'de> for SeqDeserializer {
    type Error = SerdeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}
