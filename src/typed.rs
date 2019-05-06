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
    }
}

derive_typed_ipld!(IpldNull);
derive_typed_ipld!(IpldBool);
derive_typed_ipld!(IpldInteger);
derive_typed_ipld!(IpldFloat);
derive_typed_ipld!(IpldString);
derive_typed_ipld!(IpldBytes);
derive_typed_ipld!(IpldLink);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_try_into_string() {
        let string = IpldString::from("a string".to_string());
        let ipld: Ipld = string.clone().into();
        let string2: IpldString = ipld.try_into().unwrap();
        assert_eq!(string, string2);
    }

    #[test]
    fn ipld_to_typed_ipld() {
        let boolean = IpldBool::from(true);
        let typed_ipld = TypedIpld::<IpldBool>::try_from(Ipld::from(boolean.clone())).unwrap();
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
