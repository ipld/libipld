//! Typed `Ipld` representation.
use crate::error::*;
use crate::ipld::*;
use crate::untyped::*;
use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;

/// Typed `Ipld` representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedIpld<T> {
    ty: PhantomData<T>,
    ipld: Ipld,
}

impl<T> Into<Ipld> for TypedIpld<T> {
    fn into(self) -> Ipld {
        self.ipld
    }
}

impl From<IpldString> for TypedIpld<IpldString> {
    fn from(string: IpldString) -> Self {
        Self {
            ty: PhantomData,
            ipld: Ipld::from(string),
        }
    }
}

impl Into<IpldString> for TypedIpld<IpldString> {
    fn into(self) -> IpldString {
        self.ipld.try_into().expect("cannot fail")
    }
}

impl TryFrom<Ipld> for TypedIpld<IpldString> {
    type Error = IpldTypeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let string: IpldString = ipld.try_into()?;
        Ok(Self::from(string))
    }
}

impl From<IpldBool> for TypedIpld<IpldBool> {
    fn from(boolean: IpldBool) -> Self {
        Self {
            ty: PhantomData,
            ipld: Ipld::from(boolean),
        }
    }
}

impl Into<IpldBool> for TypedIpld<IpldBool> {
    fn into(self) -> IpldBool {
        self.ipld.try_into().expect("cannot fail")
    }
}

impl TryFrom<Ipld> for TypedIpld<IpldBool> {
    type Error = IpldTypeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let boolean: IpldBool = ipld.try_into()?;
        Ok(Self::from(boolean))
    }
}

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
