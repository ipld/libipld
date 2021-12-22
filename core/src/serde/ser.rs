use std::collections::BTreeMap;
use std::convert::TryFrom;

use cid::serde::CID_SERDE_NEWTYPE_STRUCT_NAME;
use cid::Cid;
use serde::ser;

use crate::error::SerdeError;
use crate::ipld::Ipld;

/// Serialize into instances of [`crate::ipld::Ipld`].
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use libipld_core::ipld::Ipld;
/// use libipld_core::serde::to_ipld;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u8,
///     hobbies: Vec<String>,
///     is_cool: bool,
/// }
///
/// let person = Person {
///     name: "Hello World!".into(),
///     age: 52,
///     hobbies: vec!["geography".into(), "programming".into()],
///     is_cool: true,
/// };
///
/// let ipld = to_ipld(person);
/// assert!(matches!(ipld, Ok(Ipld::Map(_))));
/// ```
pub fn to_ipld<T>(value: T) -> Result<Ipld, SerdeError>
where
    T: ser::Serialize,
{
    value.serialize(Serializer)
}

impl ser::Serialize for Ipld {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match &self {
            Self::Null => serializer.serialize_none(),
            Self::Bool(bool) => serializer.serialize_bool(*bool),
            Self::Integer(i128) => serializer.serialize_i128(*i128),
            Self::Float(f64) => serializer.serialize_f64(*f64),
            Self::String(string) => serializer.serialize_str(string),
            Self::Bytes(bytes) => serializer.serialize_bytes(bytes),
            Self::List(list) => serializer.collect_seq(list),
            Self::Map(map) => serializer.collect_map(map),
            Self::Link(link) => {
                serializer.serialize_newtype_struct(CID_SERDE_NEWTYPE_STRUCT_NAME, link)
            }
        }
    }
}

struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Ipld;
    type Error = SerdeError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(i128::from(value))
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Integer(value))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Integer(value.into()))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Float(value))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Bytes(value.to_vec()))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let ipld = value.serialize(self);
        if name == CID_SERDE_NEWTYPE_STRUCT_NAME {
            if let Ok(Ipld::Bytes(bytes)) = ipld {
                let cid = Cid::try_from(bytes)
                    .map_err(|err| SerdeError(format!("Invalid CID: {}", err)))?;
                return Ok(Self::Ok::Link(cid));
            }
            // TODO vmx 2021-12-22: is `serialize_newtype_struct` called in any other cases? should
            // we perhaps error here?
        }
        ipld
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let mut values = BTreeMap::new();
        values.insert(variant.to_owned(), value.serialize(self)?);
        Ok(Self::Ok::Map(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: BTreeMap::new(),
        })
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct SerializeVec {
    vec: Vec<Ipld>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Ipld>,
}

pub struct SerializeMap {
    map: BTreeMap<String, Ipld>,
    next_key: Option<String>,
}

pub struct SerializeStructVariant {
    name: String,
    map: BTreeMap<String, Ipld>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::List(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        self.vec.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut object = BTreeMap::new();

        object.insert(self.name, Self::Ok::List(self.vec));

        Ok(Self::Ok::Map(object))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        match key.serialize(Serializer)? {
            Ipld::String(string_key) => {
                self.next_key = Some(string_key);
                Ok(())
            }
            _ => Err(SerdeError("Map keys must be strings".to_string())),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Map(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        serde::ser::SerializeMap::end(self)
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Ipld;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        self.map
            .insert(key.to_string(), value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut object = BTreeMap::new();

        object.insert(self.name, Self::Ok::Map(self.map));

        Ok(Self::Ok::Map(object))
    }
}
