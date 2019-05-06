//! `Ipld` types.
//!
//! Every `Ipld` type is required to implement `From` and `Into` for all
//! relevant Rust types.
//!
//! Every `Ipld` type implements `From<Ipld>` and `From<TypedIpld<T>>`.
pub use cid::Cid;

macro_rules! derive_from_into {
    ($ipld:ident, $rust:ty) => {
        impl From<$rust> for $ipld {
            fn from(ipld: $rust) -> Self {
                $ipld(ipld)
            }
        }

        impl Into<$rust> for $ipld {
            fn into(self) -> $rust {
                self.0
            }
        }
    };
}

/// Represents `null` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldNull;

/// Represents a `bool` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldBool(bool);
derive_from_into!(IpldBool, bool);

/// Represents an integer in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpldInteger {
    /// Represents an unsigned integer.
    U64(u64),
    /// Represents a signed integer.
    I64(i64),
}

macro_rules! derive_from_into_integer {
    ($repr:ident, $repr_ty:ty, $type:ty) => {
        impl From<$type> for IpldInteger {
            fn from(integer: $type) -> Self {
                IpldInteger::$repr(integer as $repr_ty)
            }
        }

        impl Into<$type> for IpldInteger {
            fn into(self) -> $type {
                match self {
                    IpldInteger::U64(integer) => integer as $type,
                    IpldInteger::I64(integer) => integer as $type,
                }
            }
        }
    };
}

macro_rules! derive_from_into_u64 {
    ($type:ty) => {
        derive_from_into_integer!(U64, u64, $type);
    };
}

macro_rules! derive_from_into_i64 {
    ($type:ty) => {
        derive_from_into_integer!(I64, i64, $type);
    };
}

derive_from_into_u64!(u8);
derive_from_into_u64!(u16);
derive_from_into_u64!(u32);
derive_from_into_u64!(u64);
derive_from_into_u64!(usize);
derive_from_into_i64!(i8);
derive_from_into_i64!(i16);
derive_from_into_i64!(i32);
derive_from_into_i64!(i64);
derive_from_into_i64!(isize);

/// Represents a floating point value in `Ipld`.
#[derive(Clone, Debug, PartialEq)]
pub struct IpldFloat(f64);
derive_from_into!(IpldFloat, f64);

impl From<f32> for IpldFloat {
    fn from(float: f32) -> Self {
        IpldFloat(float as f64)
    }
}

impl Into<f32> for IpldFloat {
    fn into(self) -> f32 {
        self.0 as f32
    }
}

/// Represents a `String` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldString(String);
derive_from_into!(IpldString, String);

impl From<&str> for IpldString {
    fn from(string: &str) -> Self {
        IpldString(string.to_string())
    }
}

/// Represents a sequence of bytes in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldBytes(Vec<u8>);
derive_from_into!(IpldBytes, Vec<u8>);

/// Represents a link in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldLink(Cid);
derive_from_into!(IpldLink, Cid);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_into_string() {
        let string: String = "a string".into();
        let ipld: IpldString = string.clone().into();
        let string2: String = ipld.into();
        assert_eq!(string, string2);
    }

    #[test]
    fn from_into_bool() {
        let boolean: bool = true;
        let ipld: IpldBool = boolean.into();
        let boolean2: bool = ipld.into();
        assert_eq!(boolean, boolean2);
    }
}
