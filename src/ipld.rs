//! Untyped `Ipld` representation.

use crate::error::{Error, IpldKeyTypeError, IpldTypeError};
use cid::Cid;
use core::convert::{TryFrom, TryInto};
use std::collections::HashMap;

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
    Map(HashMap<IpldKey, Ipld>),
    /// Represents a link to an Ipld node
    Link(Cid),
}

/// Ipld key
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IpldKey {
    /// Represents the absence of a value or the value undefined.
    Null,
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an integer.
    Integer(i128),
    /// Represents an UTF-8 string.
    String(String),
    /// Represents a sequence of bytes.
    Bytes(Vec<u8>),
    /// Represents a link to an Ipld node
    Link(Cid),
}

impl From<IpldKey> for Ipld {
    fn from(key: IpldKey) -> Self {
        match key {
            IpldKey::Null => Ipld::Null,
            IpldKey::Bool(b) => Ipld::Bool(b),
            IpldKey::Integer(i) => Ipld::Integer(i),
            IpldKey::String(s) => Ipld::String(s),
            IpldKey::Bytes(b) => Ipld::Bytes(b),
            IpldKey::Link(c) => Ipld::Link(c),
        }
    }
}

impl TryFrom<Ipld> for IpldKey {
    type Error = IpldKeyTypeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Null => Ok(IpldKey::Null),
            Ipld::Bool(b) => Ok(IpldKey::Bool(b)),
            Ipld::Integer(i) => Ok(IpldKey::Integer(i)),
            Ipld::String(s) => Ok(IpldKey::String(s)),
            Ipld::Bytes(b) => Ok(IpldKey::Bytes(b)),
            Ipld::Link(c) => Ok(IpldKey::Link(c)),
            _ => Err(IpldKeyTypeError),
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
            type Error = Error;

            fn try_from(ipld: $name) -> Result<$type, Self::Error> {
                match ipld {
                    $name::$enum(ty) => Ok(ty.try_into()?),
                    _ => Err(IpldTypeError::$error.into()),
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

macro_rules! derive_from_nokey {
    ($enum:ident, $type:ty) => {
        derive_from!(Ipld, $enum, $type);
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

macro_rules! derive_from_key {
    ($enum:ident, $type:ty) => {
        derive_from!(Ipld, $enum, $type);
        derive_from!(IpldKey, $enum, $type);
    };
}

// Types that can be used as keys
derive_key!(Bool, bool, NotBool);
derive_key!(Integer, i8, NotInteger);
derive_key!(Integer, i16, NotInteger);
derive_key!(Integer, i32, NotInteger);
derive_key!(Integer, i64, NotInteger);
derive_key!(Integer, i128, NotInteger);
derive_key!(Integer, u8, NotInteger);
derive_key!(Integer, u16, NotInteger);
derive_key!(Integer, u32, NotInteger);
derive_key!(Integer, u64, NotInteger);
derive_key!(String, String, NotString);
derive_key!(Bytes, Vec<u8>, NotBytes);
derive_key!(Link, Cid, NotLink);

// Additional From implementations
derive_from_key!(String, &str);
derive_from_key!(Bytes, &[u8]);
derive_from_key!(Link, &Cid);

// Types that cannot be used as keys
derive_nokey!(Float, f64, NotFloat);
derive_nokey!(List, Vec<Ipld>, NotList);
derive_nokey!(Map, HashMap<IpldKey, Ipld>, NotMap);

// Additional From implementations
derive_from_nokey!(Float, f32);
derive_from_nokey!(List, &[Ipld]);

/*
impl<T: Into<Ipld>> From<Vec<T>> for Ipld {
    fn from(vec: Vec<T>) -> Self {
        Ipld::List(vec.into_iter().map(Into::into).collect())
    }
}

impl<A: Into<IpldKey>, B: Into<Ipld>> From<HashMap<A, B>> for Ipld {
    fn from(map: HashMap<A, B>) -> Self {
        Ipld::Map(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl<A: From<IpldKey>, B: From<Ipld>> TryInto<HashMap<A, B>> for Ipld {
    type Error = Error;

    fn try_into(self) -> Result<HashMap<A, B>, Self::Error> {
        match self {
            Ipld::Map(map) => Ok(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect()),
            _ => Err(IpldTypeError::NotMap.into()),
        }
    }
}
*/

impl Ipld {
    /// Returns a bool.
    pub fn as_bool(&self) -> Option<&bool> {
        if let Ipld::Bool(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns a mutable bool.
    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        if let Ipld::Bool(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns an int.
    pub fn as_int(&self) -> Option<&i128> {
        if let Ipld::Integer(i) = self {
            Some(i)
        } else {
            None
        }
    }

    /// Returns a mutable int.
    pub fn as_int_mut(&mut self) -> Option<&mut i128> {
        if let Ipld::Integer(i) = self {
            Some(i)
        } else {
            None
        }
    }

    /// Returns a float.
    pub fn as_float(&self) -> Option<&f64> {
        if let Ipld::Float(f) = self {
            Some(f)
        } else {
            None
        }
    }

    /// Returns a mutable float.
    pub fn as_float_mut(&mut self) -> Option<&mut f64> {
        if let Ipld::Float(f) = self {
            Some(f)
        } else {
            None
        }
    }

    /// Returns a string.
    pub fn as_string(&self) -> Option<&String> {
        if let Ipld::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns a mutable string.
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let Ipld::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns a byte vec.
    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        if let Ipld::Bytes(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns a mutable byte vec.
    pub fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let Ipld::Bytes(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns a list.
    pub fn as_list(&self) -> Option<&Vec<Ipld>> {
        if let Ipld::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Returns a mutable list.
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Ipld>> {
        if let Ipld::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    /// Returns a map.
    pub fn as_map(&self) -> Option<&HashMap<IpldKey, Ipld>> {
        if let Ipld::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Returns a mutable map.
    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<IpldKey, Ipld>> {
        if let Ipld::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Returns a link.
    pub fn as_link(&self) -> Option<&Cid> {
        if let Ipld::Link(cid) = self {
            Some(cid)
        } else {
            None
        }
    }

    /// Returns a mutable link.
    pub fn as_link_mut(&mut self) -> Option<&mut Cid> {
        if let Ipld::Link(cid) = self {
            Some(cid)
        } else {
            None
        }
    }
}

/// An index into ipld
pub enum IpldIndex {
    /// An index into an ipld list.
    List(usize),
    /// An index into an ipld map.
    Map(IpldKey),
}

impl From<usize> for IpldIndex {
    fn from(index: usize) -> Self {
        Self::List(index)
    }
}

impl From<IpldKey> for IpldIndex {
    fn from(key: IpldKey) -> Self {
        Self::Map(key)
    }
}

impl From<&str> for IpldIndex {
    fn from(key: &str) -> Self {
        Self::Map(IpldKey::String(key.into()))
    }
}

/// Indexing into ipld.
pub trait IpldGet {
    /// Indexes into a map or a list.
    fn get<T: Into<IpldIndex>>(&self, index: T) -> Option<&Ipld>;
}

impl IpldGet for Ipld {
    fn get<T: Into<IpldIndex>>(&self, index: T) -> Option<&Ipld> {
        match index.into() {
            IpldIndex::List(i) => {
                if let Some(vec) = self.as_list() {
                    vec.get(i)
                } else {
                    None
                }
            }
            IpldIndex::Map(ref key) => {
                if let Some(map) = self.as_map() {
                    map.get(key)
                } else {
                    None
                }
            }
        }
    }
}

impl IpldGet for Option<&Ipld> {
    fn get<T: Into<IpldIndex>>(&self, index: T) -> Option<&Ipld> {
        self.map(|ipld| ipld.get(index)).unwrap()
    }
}

/// Mutable indexing into ipld.
pub trait IpldGetMut {
    /// Mutably indexes into a map or a list.
    fn get_mut(&mut self, index: &IpldIndex) -> Option<&mut Ipld>;
}

impl IpldGetMut for Ipld {
    fn get_mut(&mut self, index: &IpldIndex) -> Option<&mut Ipld> {
        match index {
            IpldIndex::List(i) => {
                if let Some(vec) = self.as_list_mut() {
                    vec.get_mut(*i)
                } else {
                    None
                }
            }
            IpldIndex::Map(ref key) => {
                if let Some(map) = self.as_map_mut() {
                    map.get_mut(key)
                } else {
                    None
                }
            }
        }
    }
}

/// Mutably indexing into wrappers of a mutable ipld reference.
pub trait InnerIpldGetMut<'a> {
    /// Because mut refs are not copy, we need an additional trait.
    fn get_mut(self, index: &IpldIndex) -> Option<&'a mut Ipld>;
}

impl<'a> InnerIpldGetMut<'a> for Option<&'a mut Ipld> {
    fn get_mut(self, index: &IpldIndex) -> Option<&'a mut Ipld> {
        self.map(|ipld| ipld.get_mut(index)).unwrap()
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
        assert_eq!(Ipld::Float(1.0), Ipld::from(1.0f32));
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

        let mut ipld = ipld!({});
        let map = ipld.as_map_mut().unwrap();
        map.insert("key".into(), "value".into());
        assert_eq!(ipld.get("key").unwrap(), &Ipld::String("value".into()));
    }
}
