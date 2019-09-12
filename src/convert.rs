//! Conversion to and from ipld.
use crate::error::IpldError;
use crate::ipld::{Cid, Ipld, IpldRef};
use core::borrow::Borrow;
use core::convert::TryInto;
use std::collections::BTreeMap;

/// Serialize to ipld.
pub trait ToIpld {
    /// Returns an ipld reference.
    fn to_ipld<'a>(&'a self) -> IpldRef<'a>;
}

/// Deserialize from ipld.
pub trait FromIpld: Sized {
    /// Returns an ipld error or a new instance.
    fn from_ipld(ipld: Ipld) -> Result<Self, IpldError>;
}

impl ToIpld for Ipld {
    fn to_ipld<'a>(&'a self) -> IpldRef<'a> {
        self.as_ref()
    }
}

impl FromIpld for Ipld {
    fn from_ipld(ipld: Ipld) -> Result<Self, IpldError> {
        Ok(ipld)
    }
}

macro_rules! derive_to_ipld {
    ($enum:ident, $ty:ty, $fn:ident) => {
        impl ToIpld for $ty {
            fn to_ipld<'a>(&'a self) -> IpldRef<'a> {
                IpldRef::$enum(self.$fn() as _)
            }
        }
    }
}

derive_to_ipld!(Bool, bool, clone);
derive_to_ipld!(Integer, i8, clone);
derive_to_ipld!(Integer, i16, clone);
derive_to_ipld!(Integer, i32, clone);
derive_to_ipld!(Integer, i64, clone);
derive_to_ipld!(Integer, i128, clone);
derive_to_ipld!(Integer, isize, clone);
derive_to_ipld!(Integer, u8, clone);
derive_to_ipld!(Integer, u16, clone);
derive_to_ipld!(Integer, u32, clone);
derive_to_ipld!(Integer, u64, clone);
derive_to_ipld!(Integer, usize, clone);
derive_to_ipld!(Float, f32, clone);
derive_to_ipld!(Float, f64, clone);
derive_to_ipld!(String, String, as_str);
derive_to_ipld!(String, &str, clone);
derive_to_ipld!(Bytes, Vec<u8>, as_slice);
derive_to_ipld!(Bytes, &[u8], clone);
derive_to_ipld!(List, Vec<Ipld>, as_slice);
derive_to_ipld!(Map, BTreeMap<String, Ipld>, borrow);
derive_to_ipld!(Link, Cid, borrow);
derive_to_ipld!(Link, &Cid, clone);

macro_rules! derive_from_ipld {
    ($enum:ident, $ty:ty, $err:ident) => {
        impl FromIpld for $ty {
            fn from_ipld(ipld: Ipld) -> Result<Self, IpldError> {
                if let Ipld::$enum(inner) = ipld {
                    match inner.try_into() {
                        Ok(res) => Ok(res),
                        Err(err) => Err(IpldError::Other(err.into()))
                    }
                } else {
                    Err(IpldError::$err)
                }
            }
        }
    }
}

derive_from_ipld!(Bool, bool, NotBool);
derive_from_ipld!(Integer, i8, NotInteger);
derive_from_ipld!(Integer, i16, NotInteger);
derive_from_ipld!(Integer, i32, NotInteger);
derive_from_ipld!(Integer, i64, NotInteger);
derive_from_ipld!(Integer, i128, NotInteger);
derive_from_ipld!(Integer, isize, NotInteger);
derive_from_ipld!(Integer, u8, NotInteger);
derive_from_ipld!(Integer, u16, NotInteger);
derive_from_ipld!(Integer, u32, NotInteger);
derive_from_ipld!(Integer, u64, NotInteger);
derive_from_ipld!(Integer, usize, NotInteger);
//derive_from_ipld!(Float, f32, NotFloat);
derive_from_ipld!(Float, f64, NotFloat);
derive_from_ipld!(String, String, NotString);
derive_from_ipld!(Bytes, Vec<u8>, NotBytes);
derive_from_ipld!(List, Vec<Ipld>, NotList);
derive_from_ipld!(Map, BTreeMap<String, Ipld>, NotMap);
derive_from_ipld!(Link, Cid, NotLink);

/*impl<T: ToIpld> ToIpld for Vec<T> {
    fn to_ipld<'a>(&'a self) -> IpldRef<'a> {
        IpldRef::OwnedList(self.iter().map(ToIpld::to_ipld).collect())
    }
}

impl<T: FromIpld> FromIpld for Vec<T> {
    fn from_ipld(ipld: Ipld) -> Result<Self, IpldError> {
        if let Ipld::List(list) = ipld {
            list.into_iter().map(FromIpld::from_ipld).collect()
        } else {
            Err(IpldError::NotList)
        }
    }
}

impl<V: ToIpld> ToIpld for BTreeMap<String, V> {
    fn to_ipld<'a>(&'a self) -> IpldRef<'a> {
        IpldRef::OwnedMap(self.iter().map(|(k, v)| (k.to_owned(), v.to_ipld())).collect())
    }
}

impl<V: FromIpld> FromIpld for BTreeMap<String, V> {
    fn from_ipld(ipld: Ipld) -> Result<Self, IpldError> {
        if let Ipld::Map(map) = ipld {
            let mut new_map = Self::new();
            for (k, v) in map.into_iter() {
                new_map.insert(k, V::from_ipld(v)?);
            }
            Ok(new_map)
        } else {
            Err(IpldError::NotMap)
        }
    }
}*/
