//! `Ipld` types.
//!
//! Every `Ipld` type is required to implement `From` and `Into` for all
//! relevant Rust types.
//!
//! Every `Ipld` type implements `From<Ipld>` and `From<TypedIpld<T>>`.
pub use cid::Cid;

/// Represents `null` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldNull;

/// Represents a `bool` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldBool(bool);

impl From<bool> for IpldBool {
    fn from(boolean: bool) -> Self {
        IpldBool(boolean)
    }
}

impl Into<bool> for IpldBool {
    fn into(self) -> bool {
        self.0
    }
}

/// Represents an integer in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpldInteger {
    /// Represents an unsigned integer.
    U64(u64),
    /// Represents a signed integer.
    I64(i64),
}

macro_rules! derive_ipld_integer {
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

macro_rules! derive_ipld_u64 {
    ($type:ty) => {
        derive_ipld_integer!(U64, u64, $type);
    };
}

macro_rules! derive_ipld_i64 {
    ($type:ty) => {
        derive_ipld_integer!(I64, i64, $type);
    };
}

derive_ipld_u64!(u8);
derive_ipld_u64!(u16);
derive_ipld_u64!(u32);
derive_ipld_u64!(u64);
derive_ipld_u64!(usize);
derive_ipld_i64!(i8);
derive_ipld_i64!(i16);
derive_ipld_i64!(i32);
derive_ipld_i64!(i64);
derive_ipld_i64!(isize);

/// Represents a floating point value in `Ipld`.
#[derive(Clone, Debug, PartialEq)]
pub struct IpldFloat(f64);

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

impl From<f64> for IpldFloat {
    fn from(float: f64) -> Self {
        IpldFloat(float)
    }
}

impl Into<f64> for IpldFloat {
    fn into(self) -> f64 {
        self.0
    }
}

/// Represents a `String` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldString(String);

impl From<String> for IpldString {
    fn from(string: String) -> Self {
        IpldString(string)
    }
}

impl Into<String> for IpldString {
    fn into(self) -> String {
        self.0
    }
}

impl From<&str> for IpldString {
    fn from(string: &str) -> Self {
        IpldString(string.to_string())
    }
}

/// Represents a sequence of bytes in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldBytes(Vec<u8>);

impl From<Vec<u8>> for IpldBytes {
    fn from(bytes: Vec<u8>) -> Self {
        IpldBytes(bytes)
    }
}

impl Into<Vec<u8>> for IpldBytes {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

/// Represents a link in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldLink(Cid);

impl From<Cid> for IpldLink {
    fn from(cid: Cid) -> Self {
        IpldLink(cid)
    }
}

impl Into<Cid> for IpldLink {
    fn into(self) -> Cid {
        self.0
    }
}

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
