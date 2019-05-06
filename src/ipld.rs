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
    fn from_into_bool() {
        let boolean = true;
        let ipld = IpldBool::from(boolean);
        let boolean2: bool = ipld.into();
        assert_eq!(boolean, boolean2);
    }

    #[test]
    fn from_into_integer() {
        let int: u8 = 1;
        let ipld = IpldInteger::from(int);
        let int2: u8 = ipld.into();
        assert_eq!(int, int2);
    }

    #[test]
    fn from_into_float() {
        let float: f32 = 1.0;
        let ipld = IpldFloat::from(float);
        let float2: f32 = ipld.into();
        assert_eq!(float, float2);
    }

    #[test]
    fn from_into_string() {
        let string = "a string".to_string();
        let ipld = IpldString::from(string.clone());
        let string2: String = ipld.into();
        assert_eq!(string, string2);
    }

    #[test]
    fn string_from_str() {
        let ipld = IpldString::from("a string");
        let ipld2 = IpldString::from("a string".to_string());
        assert_eq!(ipld, ipld2);
    }

    #[test]
    fn from_into_bytes() {
        let bytes: Vec<u8> = vec![0, 1, 2, 3];
        let ipld = IpldBytes::from(bytes.clone());
        let bytes2: Vec<u8> = ipld.into();
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn from_into_link() {
        let prefix = cid::Prefix {
            version: cid::Version::V0,
            codec: cid::Codec::DagProtobuf,
            mh_type: multihash::Hash::SHA2256,
            mh_len: 32,
        };
        let data = vec![0, 1, 2, 3];
        let link = Cid::new_from_prefix(&prefix, &data);
        let ipld = IpldLink::from(link.clone());
        let link2: Cid = ipld.into();
        assert_eq!(link, link2);
    }
}
