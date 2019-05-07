//! Untyped `Ipld` representation.

use crate::error::*;
use crate::ipld::*;
use core::convert::TryInto;
use std::collections::HashMap;

/// Untyped `Ipld` representation.
#[derive(Clone, Debug, PartialEq)]
pub enum Ipld {
    /// Represents the absence of a value or the value undefined.
    Null(IpldNull),
    /// Represents a boolean value.
    Bool(IpldBool),
    /// Represents an integer.
    Integer(IpldInteger),
    /// Represents a floating point value.
    Float(IpldFloat),
    /// Represents an UTF-8 string.
    String(IpldString),
    /// Represents a sequence of bytes.
    Bytes(IpldBytes),
    /// Represents a list.
    List(IpldList),
    /// Represents a map.
    Map(IpldMap),
    /// Represents a link to an Ipld node
    Link(IpldLink),
}

macro_rules! derive_ipld {
    ($enum:ident, $ipld:ty, $error:ident) => {
        impl From<$ipld> for Ipld {
            fn from(ipld: $ipld) -> Ipld {
                Ipld::$enum(ipld)
            }
        }

        impl TryInto<$ipld> for Ipld {
            type Error = IpldTypeError;

            fn try_into(self) -> Result<$ipld, Self::Error> {
                match self {
                    Ipld::$enum(ipld) => Ok(ipld),
                    _ => Err(IpldTypeError::$error),
                }
            }
        }
    };
}

derive_ipld!(Null, IpldNull, NotNull);
derive_ipld!(Bool, IpldBool, NotBool);
derive_ipld!(Integer, IpldInteger, NotInteger);
derive_ipld!(Float, IpldFloat, NotFloat);
derive_ipld!(String, IpldString, NotString);
derive_ipld!(Bytes, IpldBytes, NotBytes);
derive_ipld!(List, IpldList, NotList);
derive_ipld!(Map, IpldMap, NotMap);
derive_ipld!(Link, IpldLink, NotLink);

macro_rules! derive_from {
    ($enum: ident, $ipld:ident, $error:ident, $rust:ty) => {
        impl From<$rust> for Ipld {
            fn from(ipld: $rust) -> Self {
                Ipld::from($ipld::from(ipld))
            }
        }

        impl TryInto<$rust> for Ipld {
            type Error = IpldTypeError;

            fn try_into(self) -> Result<$rust, Self::Error> {
                match self {
                    Ipld::$enum(ipld) => Ok(ipld.into()),
                    _ => Err(IpldTypeError::$error),
                }
            }
        }
    };
}

macro_rules! derive_bool {
    ($rust:ty) => {
        derive_from!(Bool, IpldBool, NotBool, $rust);
    };
}

macro_rules! derive_integer {
    ($rust:ty) => {
        derive_from!(Integer, IpldInteger, NotInteger, $rust);
    };
}

macro_rules! derive_float {
    ($rust:ty) => {
        derive_from!(Float, IpldFloat, NotFloat, $rust);
    };
}

macro_rules! derive_string {
    ($rust:ty) => {
        derive_from!(String, IpldString, NotString, $rust);
    };
}

macro_rules! derive_bytes {
    ($rust:ty) => {
        derive_from!(Bytes, IpldBytes, NotBytes, $rust);
    };
}

macro_rules! derive_list {
    ($rust:ty) => {
        derive_from!(List, IpldList, NotList, $rust);
    };
}

macro_rules! derive_map {
    ($rust:ty) => {
        derive_from!(Map, IpldMap, NotMap, $rust);
    };
}

macro_rules! derive_link {
    ($rust:ty) => {
        derive_from!(Link, IpldLink, NotLink, $rust);
    };
}

derive_bool!(bool);
derive_integer!(u8);
derive_integer!(u16);
derive_integer!(u32);
derive_integer!(u64);
derive_integer!(usize);
derive_integer!(i8);
derive_integer!(i16);
derive_integer!(i32);
derive_integer!(i64);
derive_integer!(isize);
derive_float!(f32);
derive_float!(f64);
derive_string!(String);
derive_bytes!(Vec<u8>);
derive_list!(Vec<Ipld>);
derive_map!(HashMap<String, Ipld>);
derive_link!(Cid);

impl From<&str> for Ipld {
    fn from(string: &str) -> Self {
        Ipld::from(IpldString::from(string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipld_null_from() {
        Ipld::from(IpldNull);
    }

    #[test]
    fn ipld_bool_from() {
        Ipld::from(true);
        Ipld::from(IpldBool::from(false));
    }

    #[test]
    fn ipld_integer_from() {
        Ipld::from(1u8);
        Ipld::from(IpldInteger::from(-3i8));
    }

    #[test]
    fn ipld_float_from() {
        Ipld::from(1.0f32);
        Ipld::from(1.0f64);
        Ipld::from(IpldFloat::from(1.0f32));
    }

    #[test]
    fn ipld_string_from() {
        Ipld::from("a string");
        Ipld::from("a string".to_string());
        Ipld::from(IpldString::from("a string"));
        Ipld::from(IpldString::from("a string".to_string()));
    }

    #[test]
    fn ipld_bytes_from() {
        Ipld::from(vec![0, 1, 2, 3]);
        Ipld::from(IpldBytes::from(vec![0, 1, 2, 3]));
    }

    #[test]
    fn ipld_link_from() {
        let prefix = cid::Prefix {
            version: cid::Version::V0,
            codec: cid::Codec::DagProtobuf,
            mh_type: multihash::Hash::SHA2256,
            mh_len: 32,
        };
        let data = vec![0, 1, 2, 3];
        let link = Cid::new_from_prefix(&prefix, &data);
        Ipld::from(link.clone());
        Ipld::from(IpldLink::from(link));
    }

    #[test]
    fn from_try_into_string() {
        let ipld1 = IpldString::from("a string");
        let ipld = Ipld::from(ipld1.clone());
        let ipld2: IpldString = ipld.try_into().unwrap();
        assert_eq!(ipld1, ipld2);
    }
}
