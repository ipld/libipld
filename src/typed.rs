//! Typed `Ipld` representation.
use crate::error::*;
use crate::ipld::*;
use crate::untyped::*;
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;

/// Typed `Ipld` representation.
#[derive(Clone, Debug, PartialEq)]
pub struct TypedIpld<T> {
    ty: PhantomData<T>,
    ipld: Ipld,
}

impl<T> Into<Ipld> for TypedIpld<T> {
    fn into(self) -> Ipld {
        self.ipld
    }
}

macro_rules! derive_typed_ipld {
    ($ipld:ident) => {
        impl From<$ipld> for TypedIpld<$ipld> {
            fn from(ipld: $ipld) -> Self {
                Self {
                    ty: PhantomData,
                    ipld: Ipld::from(ipld),
                }
            }
        }

        impl Into<$ipld> for TypedIpld<$ipld> {
            fn into(self) -> $ipld {
                self.ipld.try_into().expect("cannot fail")
            }
        }

        impl TryFrom<Ipld> for TypedIpld<$ipld> {
            type Error = IpldTypeError;

            fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
                let ipld: $ipld = ipld.try_into()?;
                Ok(Self::from(ipld))
            }
        }
    };
}

derive_typed_ipld!(IpldNull);
derive_typed_ipld!(IpldBool);
derive_typed_ipld!(IpldInteger);
derive_typed_ipld!(IpldFloat);
derive_typed_ipld!(IpldString);
derive_typed_ipld!(IpldBytes);
derive_typed_ipld!(IpldLink);

macro_rules! derive_typed_from_into {
    ($ipld:ident, $rust:ty) => {
        impl From<$rust> for TypedIpld<$ipld> {
            fn from(ipld: $rust) -> Self {
                TypedIpld::from($ipld::from(ipld))
            }
        }

        impl Into<$rust> for TypedIpld<$ipld> {
            fn into(self) -> $rust {
                let ipld: $ipld = self.into();
                ipld.into()
            }
        }
    };
}

derive_typed_from_into!(IpldBool, bool);
derive_typed_from_into!(IpldInteger, u8);
derive_typed_from_into!(IpldInteger, u16);
derive_typed_from_into!(IpldInteger, u32);
derive_typed_from_into!(IpldInteger, u64);
derive_typed_from_into!(IpldInteger, usize);
derive_typed_from_into!(IpldInteger, i8);
derive_typed_from_into!(IpldInteger, i16);
derive_typed_from_into!(IpldInteger, i32);
derive_typed_from_into!(IpldInteger, i64);
derive_typed_from_into!(IpldInteger, isize);
derive_typed_from_into!(IpldFloat, f32);
derive_typed_from_into!(IpldFloat, f64);
derive_typed_from_into!(IpldString, String);
derive_typed_from_into!(IpldBytes, Vec<u8>);
derive_typed_from_into!(IpldLink, Cid);

impl From<&str> for TypedIpld<IpldString> {
    fn from(ipld: &str) -> Self {
        TypedIpld::from(IpldString::from(ipld))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipld_typed_from() {
        let _: TypedIpld<IpldBool> = TypedIpld::from(IpldBool::from(true));
        let _: TypedIpld<IpldBool> = TypedIpld::from(true);
    }

    #[test]
    fn ipld_typed_try_from() {
        let _: TypedIpld<IpldBool> = TypedIpld::try_from(Ipld::from(true)).unwrap();
    }

    #[test]
    fn ipld_typed_into() {
        let _: Ipld = TypedIpld::from(IpldBool::from(true)).into();
    }

    #[test]
    fn ipld_to_typed_ipld() {
        let boolean = IpldBool::from(true);
        let ipld: Ipld = boolean.clone().into();
        let typed_ipld = TypedIpld::<IpldBool>::try_from(ipld).unwrap();
        let boolean2: IpldBool = typed_ipld.into();
        assert_eq!(boolean, boolean2);
    }

    #[test]
    fn typed_ipld_to_ipld() {
        let typed_ipld = TypedIpld::<IpldBool>::from(IpldBool::from(true));
        let ipld: Ipld = typed_ipld.clone().into();
        let typed_ipld2 = TypedIpld::<IpldBool>::try_from(ipld).unwrap();
        assert_eq!(typed_ipld, typed_ipld2);
    }
}
