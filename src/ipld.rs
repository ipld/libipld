//! Ipld representation.

use crate::error::IpldError;
use cid::Cid;
use core::convert::{TryFrom, TryInto};
use std::collections::BTreeMap;

/// Ipld
#[derive(Clone, Debug, PartialEq)]
pub enum Ipld {
    /// Represents the absence of a value or the value undefined.
    Null,
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an integer.
    Integer(i128),
    /// Represents a floating point value.
    Float(f64),
    /// Represents an UTF-8 string.
    String(String),
    /// Represents a sequence of bytes.
    Bytes(Vec<u8>),
    /// Represents a list.
    List(Vec<Ipld>),
    /// Represents a map.
    Map(BTreeMap<IpldKey, Ipld>),
    /// Represents a link to an Ipld node
    Link(Cid),
}

/// Ipld ref
#[derive(Clone, Debug, PartialEq)]
pub enum IpldRef<'a> {
    /// Represents the absence of a value or the value undefined.
    Null,
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an integer.
    Integer(i128),
    /// Represents a floating point value.
    Float(f64),
    /// Represents an UTF-8 string.
    String(&'a str),
    /// Represents a sequence of bytes.
    Bytes(&'a [u8]),
    /// Represents a list.
    List(&'a [Ipld]),
    /// Represents an owned list.
    OwnedList(Vec<IpldRef<'a>>),
    /// Represents a map.
    Map(&'a BTreeMap<IpldKey, Ipld>),
    /// Represents an owned map.
    OwnedMap(BTreeMap<IpldKey, IpldRef<'a>>),
    /// Represents a link to an Ipld node
    Link(&'a Cid),
}

impl<'a> From<&'a Ipld> for IpldRef<'a> {
    fn from(ipld: &'a Ipld) -> Self {
        ipld.as_ref()
    }
}

impl<'a> From<IpldRef<'a>> for Ipld {
    fn from(ipld: IpldRef<'a>) -> Self {
        ipld.to_owned()
    }
}

/// Ipld key
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IpldKey {
    /// Represents an integer.
    Integer(i128),
    /// Represents an UTF-8 string.
    String(String),
    /// Represents a sequence of bytes.
    Bytes(Vec<u8>),
}

impl From<IpldKey> for Ipld {
    fn from(key: IpldKey) -> Self {
        match key {
            IpldKey::Integer(i) => Ipld::Integer(i),
            IpldKey::String(s) => Ipld::String(s),
            IpldKey::Bytes(b) => Ipld::Bytes(b),
        }
    }
}

impl TryFrom<Ipld> for IpldKey {
    type Error = IpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Integer(i) => Ok(IpldKey::Integer(i)),
            Ipld::String(s) => Ok(IpldKey::String(s)),
            Ipld::Bytes(b) => Ok(IpldKey::Bytes(b)),
            _ => Err(IpldError::NotKey),
        }
    }
}

macro_rules! derive_from {
    ($name:ident, $enum:ident, $type:ty) => {
        impl From<$type> for $name {
            fn from(ty: $type) -> $name {
                $name::$enum(ty.into())
            }
        }
    };
}

macro_rules! derive_try_from {
    ($name:ident, $enum:ident, $type:ty, $error:ident) => {
        impl TryFrom<$name> for $type {
            type Error = IpldError;

            fn try_from(ipld: $name) -> Result<$type, Self::Error> {
                match ipld {
                    $name::$enum(ty) => match ty.try_into() {
                        Ok(res) => Ok(res),
                        Err(err) => Err(IpldError::Other(err.into())),
                    },
                    _ => Err(IpldError::$error),
                }
            }
        }
    };
}

macro_rules! derive_nokey {
    ($enum:ident, $type:ty, $error:ident) => {
        derive_from!(Ipld, $enum, $type);
        derive_try_from!(Ipld, $enum, $type, $error);
    };
}

macro_rules! derive_key {
    ($enum:ident, $type:ty, $error:ident) => {
        derive_from!(Ipld, $enum, $type);
        derive_from!(IpldKey, $enum, $type);
        derive_try_from!(Ipld, $enum, $type, $error);
        derive_try_from!(IpldKey, $enum, $type, $error);
    };
}

macro_rules! derive_from_ref {
    ($name:ident, $enum:ident, $type:ty) => {
        impl<'a> From<&'a $type> for $name<'a> {
            fn from(ty: &'a $type) -> $name<'a> {
                $name::$enum(ty.into())
            }
        }
    };
}

macro_rules! derive_ref_key {
    ($enum:ident, $type:ty) => {
        derive_from_ref!(IpldRef, $enum, $type);
        derive_from!(Ipld, $enum, &$type);
        derive_from!(IpldKey, $enum, &$type);
    };
}

macro_rules! derive_ref_nokey {
    ($enum:ident, $type:ty) => {
        derive_from_ref!(IpldRef, $enum, $type);
        derive_from!(Ipld, $enum, &$type);
    };
}

macro_rules! derive_from_ref_copy {
    ($name:ident, $enum:ident, $type:ty) => {
        impl<'a> From<&'a $type> for $name<'a> {
            fn from(ty: &'a $type) -> $name<'a> {
                $name::$enum((*ty).into())
            }
        }
    };
}

macro_rules! derive_ref_copy {
    ($enum:ident, $type:ty) => {
        derive_from_ref_copy!(IpldRef, $enum, $type);
    };
}

macro_rules! derive_ref {
    ($enum:ident, $type:ty) => {
        derive_from_ref!(IpldRef, $enum, $type);
    };
}

derive_nokey!(Bool, bool, NotBool);
derive_key!(Integer, i8, NotInteger);
derive_key!(Integer, i16, NotInteger);
derive_key!(Integer, i32, NotInteger);
derive_key!(Integer, i64, NotInteger);
derive_key!(Integer, i128, NotInteger);
derive_key!(Integer, u8, NotInteger);
derive_key!(Integer, u16, NotInteger);
derive_key!(Integer, u32, NotInteger);
derive_key!(Integer, u64, NotInteger);
//derive_nokey!(Float, f32, NotFloat);
derive_nokey!(Float, f64, NotFloat);
derive_key!(String, String, NotString);
derive_key!(Bytes, Vec<u8>, NotBytes);
derive_nokey!(List, Vec<Ipld>, NotList);
derive_nokey!(Map, BTreeMap<IpldKey, Ipld>, NotMap);
derive_nokey!(Link, Cid, NotLink);

derive_ref_copy!(Bool, bool);
derive_ref_copy!(Integer, i8);
derive_ref_copy!(Integer, i16);
derive_ref_copy!(Integer, i32);
derive_ref_copy!(Integer, i64);
derive_ref_copy!(Integer, u8);
derive_ref_copy!(Integer, u16);
derive_ref_copy!(Integer, u32);
derive_ref_copy!(Integer, u64);
derive_ref_copy!(Float, f64);
derive_ref_key!(String, str);
derive_ref_key!(Bytes, [u8]);
derive_ref_nokey!(List, [Ipld]);
derive_ref!(Map, BTreeMap<IpldKey, Ipld>);
derive_ref_nokey!(Link, Cid);

impl<'a> From<&'a String> for IpldRef<'a> {
    fn from(s: &'a String) -> Self {
        IpldRef::String(s.as_str())
    }
}

impl<'a> From<&'a Vec<u8>> for IpldRef<'a> {
    fn from(b: &'a Vec<u8>) -> Self {
        IpldRef::Bytes(b.as_slice())
    }
}

impl<'a> From<&'a Vec<Ipld>> for IpldRef<'a> {
    fn from(l: &'a Vec<Ipld>) -> Self {
        IpldRef::List(l.as_slice())
    }
}

impl<'a> From<Vec<IpldRef<'a>>> for IpldRef<'a> {
    fn from(l: Vec<IpldRef<'a>>) -> Self {
        IpldRef::OwnedList(l)
    }
}

impl<'a> From<BTreeMap<IpldKey, IpldRef<'a>>> for IpldRef<'a> {
    fn from(m: BTreeMap<IpldKey, IpldRef<'a>>) -> Self {
        IpldRef::OwnedMap(m)
    }
}

/// An index into ipld
pub enum IpldIndex<'a> {
    /// An index into an ipld list.
    List(usize),
    /// An owned index into an ipld map.
    Map(IpldKey),
    /// An index into an ipld map.
    MapRef(&'a IpldKey),
}

impl<'a> From<usize> for IpldIndex<'a> {
    fn from(index: usize) -> Self {
        Self::List(index)
    }
}

impl<'a> From<IpldKey> for IpldIndex<'a> {
    fn from(key: IpldKey) -> Self {
        Self::Map(key)
    }
}

impl<'a> From<&'a IpldKey> for IpldIndex<'a> {
    fn from(key: &'a IpldKey) -> Self {
        Self::MapRef(key)
    }
}

impl<'a> From<&str> for IpldIndex<'a> {
    fn from(key: &str) -> Self {
        Self::Map(IpldKey::String(key.into()))
    }
}

impl Ipld {
    /// Indexes into a ipld list or map.
    pub fn get<'a, T: Into<IpldIndex<'a>>>(&self, index: T) -> Option<&Ipld> {
        match self {
            Ipld::List(l) => match index.into() {
                IpldIndex::List(i) => l.get(i),
                _ => None,
            },
            Ipld::Map(m) => match index.into() {
                IpldIndex::Map(ref key) => m.get(key),
                IpldIndex::MapRef(key) => m.get(key),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns a ipld reference.
    pub fn as_ref<'a>(&'a self) -> IpldRef<'a> {
        match self {
            Ipld::Null => IpldRef::Null,
            Ipld::Bool(b) => IpldRef::Bool(*b),
            Ipld::Integer(i) => IpldRef::Integer(*i),
            Ipld::Float(f) => IpldRef::Float(*f),
            Ipld::String(ref s) => IpldRef::String(s),
            Ipld::Bytes(ref b) => IpldRef::Bytes(b),
            Ipld::List(ref l) => IpldRef::List(l),
            Ipld::Map(ref m) => IpldRef::Map(m),
            Ipld::Link(ref c) => IpldRef::Link(c),
        }
    }
}

impl<'a> IpldRef<'a> {
    /// Turns an ipld reference into an owned ipld.
    pub fn to_owned(self) -> Ipld {
        match self {
            IpldRef::Null => Ipld::Null,
            IpldRef::Bool(b) => Ipld::Bool(b),
            IpldRef::Integer(i) => Ipld::Integer(i),
            IpldRef::Float(f) => Ipld::Float(f),
            IpldRef::String(s) => Ipld::String(s.to_string()),
            IpldRef::Bytes(b) => Ipld::Bytes(b.to_vec()),
            IpldRef::List(l) => Ipld::List(l.to_vec()),
            IpldRef::OwnedList(l) => Ipld::List(l.into_iter().map(|v| v.to_owned()).collect()),
            IpldRef::Map(m) => Ipld::Map((*m).clone()),
            IpldRef::OwnedMap(m) => {
                Ipld::Map(m.into_iter().map(|(k, v)| (k, v.to_owned())).collect())
            }
            IpldRef::Link(c) => Ipld::Link((*c).clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{Hash, Sha2_256};
    use crate::ipld;

    #[test]
    fn ipld_bool_from() {
        assert_eq!(Ipld::Bool(true), Ipld::from(true));
        assert_eq!(Ipld::Bool(false), Ipld::from(false));
    }

    #[test]
    fn ipld_integer_from() {
        assert_eq!(Ipld::Integer(1), Ipld::from(1i8));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i16));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i32));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i64));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i128));

        assert_eq!(Ipld::Integer(1), Ipld::from(1u8));
        assert_eq!(Ipld::Integer(1), Ipld::from(1u16));
        assert_eq!(Ipld::Integer(1), Ipld::from(1u32));
        assert_eq!(Ipld::Integer(1), Ipld::from(1u64));
    }

    #[test]
    fn ipld_float_from() {
        //assert_eq!(Ipld::Float(1.0), Ipld::from(1.0f32));
        assert_eq!(Ipld::Float(1.0), Ipld::from(1.0f64));
    }

    #[test]
    fn ipld_string_from() {
        assert_eq!(Ipld::String("a string".into()), Ipld::from("a string"));
        assert_eq!(
            Ipld::String("a string".into()),
            Ipld::from("a string".to_string())
        );
    }

    #[test]
    fn ipld_bytes_from() {
        assert_eq!(Ipld::Bytes(vec![0, 1, 2, 3]), Ipld::from(&[0, 1, 2, 3][..]));
        assert_eq!(Ipld::Bytes(vec![0, 1, 2, 3]), Ipld::from(vec![0, 1, 2, 3]));
    }

    #[test]
    fn ipld_link_from() {
        let data = vec![0, 1, 2, 3];
        let hash = Sha2_256::digest(&data);
        let cid = Cid::new_v0(hash).unwrap();
        assert_eq!(Ipld::Link(cid.clone()), Ipld::from(cid));
    }

    #[test]
    fn from_try_into_string() {
        let string1 = "hello world".to_string();
        let string2: String = Ipld::from(string1.clone()).try_into().unwrap();
        assert_eq!(string1, string2);
    }

    #[test]
    fn index() {
        let ipld = ipld!([0, 1, 2]);
        assert_eq!(ipld.get(0).unwrap(), &Ipld::Integer(0));
        assert_eq!(ipld.get(1).unwrap(), &Ipld::Integer(1));
        assert_eq!(ipld.get(2).unwrap(), &Ipld::Integer(2));

        let ipld = ipld!({
            "a": 0,
            "b": 1,
            "c": 2,
        });
        assert_eq!(ipld.get("a").unwrap(), &Ipld::Integer(0));
    }
}
