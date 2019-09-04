//! Untyped `Ipld` representation.

use crate::error::{Error, IpldTypeError};
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
    Map(HashMap<String, Ipld>),
    /// Represents a link to an Ipld node
    Link(Cid),
}

macro_rules! derive_from {
    ($enum:ident, $type:ty) => {
        impl From<$type> for Ipld {
            fn from(ty: $type) -> Ipld {
                Ipld::$enum(ty.into())
            }
        }
    };
}

macro_rules! derive_try_from {
    ($enum:ident, $type:ty, $error:ident) => {
        impl TryFrom<Ipld> for $type {
            type Error = Error;

            fn try_from(ipld: Ipld) -> Result<$type, Self::Error> {
                match ipld {
                    Ipld::$enum(ty) => Ok(ty.try_into()?),
                    _ => Err(IpldTypeError::$error.into()),
                }
            }
        }
    };
}

macro_rules! derive_ipld {
    ($enum:ident, $type:ty, $error:ident) => {
        derive_from!($enum, $type);
        derive_try_from!($enum, $type, $error);
    };
}

derive_ipld!(Bool, bool, NotBool);
derive_ipld!(Integer, i8, NotInteger);
derive_ipld!(Integer, i16, NotInteger);
derive_ipld!(Integer, i32, NotInteger);
derive_ipld!(Integer, i64, NotInteger);
derive_ipld!(Integer, i128, NotInteger);
derive_ipld!(Integer, u8, NotInteger);
derive_ipld!(Integer, u16, NotInteger);
derive_ipld!(Integer, u32, NotInteger);
derive_ipld!(Integer, u64, NotInteger);
derive_ipld!(Float, f64, NotFloat);
derive_ipld!(String, String, NotString);
derive_ipld!(Bytes, Vec<u8>, NotBytes);
derive_ipld!(List, Vec<Ipld>, NotList);
derive_ipld!(Map, HashMap<String, Ipld>, NotMap);
derive_ipld!(Link, Cid, NotLink);

derive_from!(Float, f32);
derive_from!(String, &str);
derive_from!(Bytes, &[u8]);
derive_from!(List, &[Ipld]);

impl From<&Cid> for Ipld {
    fn from(cid: &Cid) -> Self {
        Ipld::Link(cid.to_owned())
    }
}

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
    pub fn as_map(&self) -> Option<&HashMap<String, Ipld>> {
        if let Ipld::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Returns a mutable map.
    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, Ipld>> {
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
pub enum IpldIndex<'a> {
    /// An index into an ipld list.
    List(usize),
    /// An index into an ipld map.
    Map(&'a str),
}

impl From<usize> for IpldIndex<'_> {
    fn from(index: usize) -> Self {
        Self::List(index)
    }
}

impl<'a> From<&'a str> for IpldIndex<'a> {
    fn from(key: &'a str) -> Self {
        Self::Map(key)
    }
}

impl Ipld {
    /// Indexes into a map or a list.
    pub fn get<'a, T: Into<IpldIndex<'a>>>(&self, index: T) -> Option<&Ipld> {
        match index.into() {
            IpldIndex::List(i) => {
                if let Some(vec) = self.as_list() {
                    vec.get(i)
                } else {
                    None
                }
            }
            IpldIndex::Map(s) => {
                if let Some(map) = self.as_map() {
                    map.get(s)
                } else {
                    None
                }
            }
        }
    }

    /// Mutably indexes into a map or a list.
    pub fn get_mut<'a, T: Into<IpldIndex<'a>>>(&mut self, index: T) -> Option<&mut Ipld> {
        match index.into() {
            IpldIndex::List(i) => {
                if let Some(vec) = self.as_list_mut() {
                    vec.get_mut(i)
                } else {
                    None
                }
            }
            IpldIndex::Map(s) => {
                if let Some(map) = self.as_map_mut() {
                    map.get_mut(s)
                } else {
                    None
                }
            }
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
