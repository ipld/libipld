//! Untyped `Ipld` representation.

use crate::error::*;
use crate::ipld::*;
use core::convert::TryInto;

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
    // /// Represents a list.
    // List(IpldList),
    // /// Represents a map.
    // Map(IpldMap),
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
    fn ipld_string_from() {
        Ipld::from("a string");
        Ipld::from("a string".to_string());
        Ipld::from(IpldString::from("a string"));
        Ipld::from(IpldString::from("a string".to_string()));
    }

    #[test]
    fn from_try_into_string() {
        let string = IpldString::from("a string".to_string());
        let ipld: Ipld = string.clone().into();
        let string2: IpldString = ipld.try_into().unwrap();
        assert_eq!(string, string2);
    }
}
