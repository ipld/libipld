//! An IPLD type that borrows most of it's contents from an underlying type.
//!
//! IPLD introduces two fundamental concepts, Data Types and Schemas/Representations.
//! In short, Data Types are a small set of common types (lists, maps, links, etc)
//! necessary for generally modelling Linked Data and can be serialized/deserialized
//! by an IPLD CodecExt. However, more advanced types might benefit from having schemas,
//! alternate serialization/deserialization behaviour, or runtime dependencies to
//! aid in verification when encoding/decoding the type to/from raw blocks.
//!
//! This type fulfills the role of a low-allocation mapping between IPLD Schemas
//! /Representations and underlying Codecs by actually providing two mappings:
//!     - one from IPLD Data type <-> IPLD CodecExt (via Serde data model)
//!     - one from Rust types, schemas & representations <-> IPLD (via `TryFrom`)
//! In the first case, Serde only borrows from the type on serialization, and
//! whenever possible provides borrowed types on deserialization. Likewise,
//! `TryFrom<Ipld>` and `TryInto<Ipld>` are implemented to borrow
//! many of their fields or lazily iterate over them.

use crate::dev::*;
use serde::{
    de::{self, Visitor},
    serde_if_integer128,
};
use std::{
    cell::RefCell,
    collections::btree_map::{BTreeMap, Iter as BTreeMapIter},
    convert::TryFrom,
    fmt,
    marker::PhantomData,
    slice::Iter as SliceIter,
    vec::IntoIter as VecIter,
};

/// Ipld that borrows from an underlying type.
#[derive(Clone, Debug)]
pub enum Ipld<'a, C>
where
    C: CodecExt,
{
    /// Represents the absence of a value or the value undefined.
    Null(PhantomData<C>),
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an i8.
    Int8(i8),
    /// Represents an i16.
    Int16(i16),
    /// Represents an i32.
    Int32(i32),
    /// Represents an i64.
    Int64(i64),
    /// Represents an i128.
    Int128(i128),
    /// Represents an u8.
    Uint8(u8),
    /// Represents an u16.
    Uint16(u16),
    /// Represents an u32.
    Uint32(u32),
    /// Represents an u64.
    Uint64(u64),
    /// Represents an u128.
    Uint128(u128),
    /// Represents an f32.
    Float32(f32),
    /// Represents an f64.
    Float64(f64),
    /// Represents an UTF-8 string.
    String(&'a str),
    /// Represents a sequence of bytes.
    Bytes(&'a [u8]),
    /// Represents a list.
    List(IpldListIter<'a, C>),
    /// Represents a map.
    Map(IpldMapIter<'a, C>),
    /// Represents a link to an Ipld node
    Link(Cid),
}

// Iters

/// Wrapper around `Iterator`s commonly used with IPLD lists.
///
/// Uses `RefCell` to bypass serde's strict borrowing requirements.
#[derive(Clone, Debug)]
pub enum IpldListIter<'a, C: CodecExt> {
    Slice(RefCell<SliceIter<'a, Ipld<'a, C>>>),
    Vec(RefCell<VecIter<Ipld<'a, C>>>),
}

impl<'a, C> From<SliceIter<'a, Ipld<'a, C>>> for IpldListIter<'a, C>
where
    C: CodecExt,
{
    fn from(iter: SliceIter<'a, Ipld<'a, C>>) -> Self {
        IpldListIter::Slice(RefCell::new(iter))
    }
}

impl<'a, C> From<VecIter<Ipld<'a, C>>> for IpldListIter<'a, C>
where
    C: CodecExt,
{
    fn from(iter: VecIter<Ipld<'a, C>>) -> Self {
        IpldListIter::Vec(RefCell::new(iter))
    }
}

/// Wrapper around `Iterator`s commonly used with IPLD maps.
///
/// Uses `RefCell` to bypass serde's strict borrowing requirements.
#[derive(Clone, Debug)]
pub enum IpldMapIter<'a, C: CodecExt> {
    Vec(RefCell<VecIter<(&'a str, Ipld<'a, C>)>>),
}

impl<'a, C> From<VecIter<(&'a str, Ipld<'a, C>)>> for IpldMapIter<'a, C>
where
    C: CodecExt,
{
    fn from(iter: VecIter<(&'a str, Ipld<'a, C>)>) -> Self {
        IpldMapIter::Vec(RefCell::new(iter))
    }
}

// Serde

impl<'a, C> Serialize for Ipld<'a, C>
where
    C: CodecExt,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Ipld::Null(_) => serializer.serialize_none(),
            Ipld::Bool(b) => serializer.serialize_bool(*b),
            Ipld::Int8(n) => serializer.serialize_i8(*n),
            Ipld::Int16(n) => serializer.serialize_i16(*n),
            Ipld::Int32(n) => serializer.serialize_i32(*n),
            Ipld::Int64(n) => serializer.serialize_i64(*n),
            Ipld::Int128(n) => serializer.serialize_i128(*n),
            Ipld::Uint8(n) => serializer.serialize_u8(*n),
            Ipld::Uint16(n) => serializer.serialize_u16(*n),
            Ipld::Uint32(n) => serializer.serialize_u32(*n),
            Ipld::Uint64(n) => serializer.serialize_u64(*n),
            Ipld::Uint128(n) => serializer.serialize_u128(*n),
            Ipld::Float32(n) => serializer.serialize_f32(*n),
            Ipld::Float64(n) => serializer.serialize_f64(*n),
            Ipld::String(s) => serializer.serialize_str(s),
            Ipld::Bytes(b) => C::serialize_bytes(*b, serializer),
            Ipld::List(list_iter) => match list_iter {
                IpldListIter::Slice(iter) => {
                    serializer.collect_seq(&mut *(iter.borrow_mut()))
                }
                IpldListIter::Vec(iter) => {
                    serializer.collect_seq(&mut *(iter.borrow_mut()))
                }
            },
            Ipld::Map(map_iter) => match map_iter {
                IpldMapIter::Vec(iter) => serializer.collect_map(&mut *(iter.borrow_mut())),
            },
            Ipld::Link(cid) => C::serialize_link(cid, serializer),
        }
    }
}

impl<'de, C> Deserialize<'de> for Ipld<'de, C>
where
    C: CodecExt,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IpldVisitor(PhantomData))
    }
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! visit {
    ($type:ty : $visit_fn:ident $member:ident) => {
        #[inline]
        fn $visit_fn<E>(self, value: $type) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ipld::$member(value))
        }
    };
}

/// `Visitor` for `Deserialize`ing a `Ipld`.
struct IpldVisitor<C>(PhantomData<C>);

impl<'de, C> Visitor<'de> for IpldVisitor<C>
where
    C: 'de + CodecExt,
{
    type Value = Ipld<'de, C>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an IPLD dag")
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Null(PhantomData))
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Null(PhantomData))
    }

    visit!(bool : visit_bool Bool);
    visit!(i8 : visit_i8 Int8);
    visit!(i16 : visit_i16 Int16);
    visit!(i32 : visit_i32 Int32);
    visit!(i64 : visit_i64 Int64);
    visit!(u8 : visit_u8 Uint8);
    visit!(u16 : visit_u16 Uint16);
    visit!(u32 : visit_u32 Uint32);
    visit!(u64 : visit_u64 Uint64);
    visit!(f32 : visit_f32 Float32);
    visit!(f64 : visit_f64 Float64);

    serde_if_integer128! {
        visit!(i128 : visit_i128 Int128);
        visit!(u128 : visit_u128 Uint128);
    }

    visit!(&'de str : visit_borrowed_str String);
    visit!(&'de [u8] : visit_borrowed_bytes Bytes);

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        C::deserialize_unknown(deserializer, self)
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec = if let Some(len) = seq.size_hint() {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        };

        while let Some(ipld) = seq.next_element()? {
            vec.push(ipld);
        }
        Ok(Ipld::List(IpldListIter::from(
            vec.into_iter(),
        )))
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut vec = if let Some(len) = map.size_hint() {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        };

        while let Some(ipld) = map.next_entry()? {
            vec.push(ipld);
        }
        Ok(Ipld::Map(IpldMapIter::from(
            vec.into_iter(),
        )))
    }
}

impl<'de, C> crate::dev::IpldVisitor<'de> for IpldVisitor<C>
where
    C: 'de + CodecExt,
{
    fn visit_link<E>(self, cid: Cid) -> Result<<Self as Visitor<'de>>::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Link(cid))
    }
}
