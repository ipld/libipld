use alloc::{
  borrow::ToOwned,
  format,
  string::ToString,
  vec::Vec,
};
use core::convert::TryFrom;

use cid::{
  serde::CID_SERDE_PRIVATE_IDENTIFIER,
  Cid,
};
use serde::ser;

use crate::{
  error::SerdeError,
  ipld::Ipld,
};

/// Serialize into instances of [`crate::ipld::Ipld`].
///
/// All Rust types can be serialized to [`crate::ipld::Ipld`], here is a list of
/// how they are converted:
///
///  - bool -> `Ipld::Bool`
///  - i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize ->
///    `Ipld::Integer`
///  - f32, f64 -> `Ipld::Float`, but `f32::NaN` and `f64::NaN` are not
///    serializable and will error
///  - char, String -> `Ipld::String`
///  - slices -> `Ipld::List`
///  - struct
///    - struct -> `Ipld::List`
///    - newtype struct -> the value the struct wraps
///    - tuple struct -> `Ipld::List`
///    - unit struct -> the empty `Ipld::List`
///  - enum:
///    - unit variant ->  indexed Ipld::List,
///    - newtype variant -> indexed Ipld::List, `[idx, value]`
///    - tuple variant -> `indexed Ipld::List, `[idx, value0, value1, ...,
///      valueN]`
///    - struct variant -> `indexed Ipld::List, `[idx, value0, value1, ...,
///      valueN]`
///  - unit (`()`) -> an empty Ipld::List
///
/// There are also common compound types that are supported:
///
///  - [`std::option::Option`] -> eithe `Ipld::Null` or the value
///  - [`serde_bytes::ByteBuf`] -> `Ipld::Bytes`
///  - lists (like e.g. [`std::vec::Vec`]) -> `Ipld::List`
///  - maps (like e.g. [`std::collections::BTreeMap`]) -> `Ipld::List` of key
///  value pairs
///  - [`cid::Cid`] -> `Ipld::Link`
///
///
/// # Example
///
/// ```
/// use libipld_core::{
///   ipld::Ipld,
///   serde::to_ipld,
/// };
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///   name: String,
///   age: u8,
///   hobbies: Vec<String>,
///   is_cool: bool,
/// }
///
/// let person = Person {
///   name: "Hello World!".into(),
///   age: 52,
///   hobbies: vec!["geography".into(), "programming".into()],
///   is_cool: true,
/// };
///
/// let ipld = to_ipld(person);
/// assert!(matches!(ipld, Ok(Ipld::List(_))));
pub fn to_ipld<T>(value: T) -> Result<Ipld, SerdeError>
where T: ser::Serialize {
  value.serialize(&Serializer)
}

impl ser::Serialize for Ipld {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: ser::Serializer {
    match &self {
      Self::Null => serializer.serialize_none(),
      Self::Bool(value) => serializer.serialize_bool(*value),
      Self::Integer(value) => serializer.serialize_i128(*value),
      Self::Float(value) => serializer.serialize_f64(*value),
      Self::String(value) => serializer.serialize_str(value),
      Self::Bytes(value) => serializer.serialize_bytes(value),
      Self::List(value) => serializer.collect_seq(value),
      Self::Map(value) => serializer.collect_seq(value),
      Self::Link(value) => value.serialize(serializer),
    }
  }
}

struct Serializer;

pub struct StructSerializer<'a> {
  ser: &'a Serializer,
  vec: Vec<Ipld>,
  variant_index: u32,
}

impl<'a> serde::Serializer for &'a Serializer {
  type Error = SerdeError;
  type Ok = Ipld;
  type SerializeMap = SerializeMap;
  type SerializeSeq = SerializeVec;
  type SerializeStruct = StructSerializer<'a>;
  type SerializeStructVariant = StructSerializer<'a>;
  type SerializeTuple = SerializeVec;
  type SerializeTupleStruct = SerializeVec;
  type SerializeTupleVariant = SerializeTupleVariant;

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
    self.serialize_i128(value.into())
  }

  #[inline]
  fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
    self.serialize_i128(value.into())
  }

  #[inline]
  fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
    self.serialize_i128(value.into())
  }

  #[inline]
  fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
    self.serialize_i128(value.into())
  }

  #[inline]
  fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
    if value.is_nan() || value.is_infinite() {
      Err(ser::Error::custom("Cannot serialize NaN or infinity for f32"))
    }
    else {
      self.serialize_f64(f64::from(value))
    }
  }

  #[inline]
  fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
    if value.is_nan() || value.is_infinite() {
      Err(ser::Error::custom("Cannot serialize NaN or infinity for f64"))
    }
    else {
      Ok(Self::Ok::Float(value))
    }
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
    Ok(Self::Ok::List(vec![]))
  }

  #[inline]
  fn serialize_unit_struct(
    self,
    _name: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    Ok(Self::Ok::List(vec![]))
  }

  #[inline]
  fn serialize_unit_variant(
    self,
    _name: &'static str,
    variant_index: u32,
    _variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    let idx = self.serialize_u32(variant_index)?;
    Ok(Self::Ok::List(vec![idx]))
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
    if name == CID_SERDE_PRIVATE_IDENTIFIER {
      if let Ok(Ipld::Bytes(bytes)) = ipld {
        let cid = Cid::try_from(bytes)
          .map_err(|err| ser::Error::custom(format!("Invalid CID: {}", err)))?;
        return Ok(Self::Ok::Link(cid));
      }
    }
    ipld
  }

  fn serialize_newtype_variant<T: ?Sized>(
    self,
    _name: &'static str,
    variant_index: u32,
    _variant: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    let values =
      Vec::from([self.serialize_u32(variant_index)?, value.serialize(self)?]);
    Ok(Self::Ok::List(values))
  }

  #[inline]
  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    Ok(Self::Ok::Null)
  }

  #[inline]
  fn serialize_some<T: ?Sized>(
    self,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    value.serialize(self)
  }

  fn serialize_seq(
    self,
    len: Option<usize>,
  ) -> Result<Self::SerializeSeq, Self::Error> {
    Ok(SerializeVec { vec: Vec::with_capacity(len.unwrap_or(0)) })
  }

  fn serialize_tuple(
    self,
    len: usize,
  ) -> Result<Self::SerializeTuple, Self::Error> {
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
    variant_index: u32,
    _variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    Ok(SerializeTupleVariant {
      idx: variant_index,
      vec: Vec::with_capacity(len),
    })
  }

  fn serialize_map(
    self,
    _len: Option<usize>,
  ) -> Result<Self::SerializeMap, Self::Error> {
    Ok(SerializeMap { vec: Vec::new(), next_key: None })
  }

  fn serialize_struct(
    self,
    _name: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeStruct, Self::Error> {
    Ok(StructSerializer { ser: &self, vec: Vec::new(), variant_index: 0 })
  }

  fn serialize_struct_variant(
    self,
    _name: &'static str,
    variant_index: u32,
    _variant: &'static str,
    _len: usize,
  ) -> Result<Self::SerializeStructVariant, Self::Error> {
    Ok(StructSerializer { ser: &self, vec: Vec::new(), variant_index })
  }

  #[inline]
  fn is_human_readable(&self) -> bool { false }
}

pub struct SerializeVec {
  vec: Vec<Ipld>,
}

pub struct SerializeTupleVariant {
  idx: u32,
  vec: Vec<Ipld>,
}

pub struct SerializeMap {
  vec: Vec<Ipld>,
  next_key: Option<Ipld>,
}

impl ser::SerializeSeq for SerializeVec {
  type Error = SerdeError;
  type Ok = Ipld;

  fn serialize_element<T: ?Sized>(
    &mut self,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    self.vec.push(value.serialize(&Serializer)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> { Ok(Self::Ok::List(self.vec)) }
}

impl ser::SerializeTuple for SerializeVec {
  type Error = SerdeError;
  type Ok = Ipld;

  fn serialize_element<T: ?Sized>(
    &mut self,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    ser::SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> { ser::SerializeSeq::end(self) }
}

impl ser::SerializeTupleStruct for SerializeVec {
  type Error = SerdeError;
  type Ok = Ipld;

  fn serialize_field<T: ?Sized>(
    &mut self,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    ser::SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> { ser::SerializeSeq::end(self) }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
  type Error = SerdeError;
  type Ok = Ipld;

  fn serialize_field<T: ?Sized>(
    &mut self,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    self.vec.push(value.serialize(&Serializer)?);
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    let mut vec = Vec::new();
    let mut args = self.vec.clone();
    vec.push(Ipld::Integer(self.idx.clone() as i128));
    vec.append(&mut args);
    Ok(Ipld::List(vec))
  }
}

impl ser::SerializeMap for SerializeMap {
  type Error = SerdeError;
  type Ok = Ipld;

  fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
  where T: ser::Serialize {
    match key.serialize(&Serializer)? {
      key => {
        self.next_key = Some(key);
        Ok(())
      }
    }
  }

  fn serialize_value<T: ?Sized>(
    &mut self,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    let key = self.next_key.take();
    // Panic because this indicates a bug in the program rather than an
    // expected failure.
    let key = key.expect("serialize_value called before serialize_key");
    self.vec.push(Ipld::List(vec![key, value.serialize(&Serializer)?]));
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> { Ok(Self::Ok::List(self.vec)) }
}

impl<'a> StructSerializer<'a> {
  #[inline]
  fn serialize_field_inner<T>(
    &mut self,
    _: &'static str,
    value: &T,
  ) -> Result<(), SerdeError>
  where
    T: ?Sized + ser::Serialize,
  {
    let val = value.serialize(self.ser)?;
    self.vec.push(val);
    Ok(())
  }

  #[inline]
  fn skip_field_inner(&mut self, _: &'static str) -> Result<(), SerdeError> {
    Ok(())
  }

  #[inline]
  fn end_inner(self) -> Result<Vec<Ipld>, SerdeError> { Ok(self.vec) }
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
  type Error = SerdeError;
  type Ok = Ipld;

  #[inline]
  fn serialize_field<T: ?Sized>(
    &mut self,
    key: &'static str,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    self.serialize_field_inner(key, value)?;
    Ok(())
  }

  #[inline]
  fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
    self.skip_field_inner(key)
  }

  #[inline]
  fn end(self) -> Result<Self::Ok, Self::Error> {
    let x = self.end_inner()?;
    Ok(Ipld::List(x))
  }
}

impl<'a> ser::SerializeStructVariant for StructSerializer<'a> {
  type Error = SerdeError;
  type Ok = Ipld;

  fn serialize_field<T: ?Sized>(
    &mut self,
    key: &'static str,
    value: &T,
  ) -> Result<(), Self::Error>
  where
    T: ser::Serialize,
  {
    self.serialize_field_inner(key, value)?;
    Ok(())
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    let mut vec = Vec::new();
    vec.push(Ipld::Integer(self.variant_index.clone() as i128));
    let mut args = self.end_inner()?;
    vec.append(&mut args);
    Ok(Ipld::List(vec))
  }
}
