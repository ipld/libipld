use core::convert::{TryFrom, TryInto};
use core::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq)]
struct IpldString(String);

impl From<String> for IpldString {
    fn from(string: String) -> IpldString {
        IpldString(string)
    }
}

impl Into<String> for IpldString {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IpldBool(bool);

impl From<bool> for IpldBool {
    fn from(boolean: bool) -> IpldBool {
        IpldBool(boolean)
    }
}

impl Into<bool> for IpldBool {
    fn into(self) -> bool {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ipld {
    String(IpldString),
    Bool(IpldBool),
}

impl From<IpldString> for Ipld {
    fn from(string: IpldString) -> Ipld {
        Ipld::String(string)
    }
}

impl TryInto<IpldString> for Ipld {
    type Error = IpldTypeError;

    fn try_into(self) -> Result<IpldString, Self::Error> {
        match self {
            Ipld::String(string) => Ok(string),
            _ => Err(IpldTypeError::NotAString),
        }
    }
}

impl From<IpldBool> for Ipld {
    fn from(boolean: IpldBool) -> Ipld {
        Ipld::Bool(boolean)
    }
}

impl TryInto<IpldBool> for Ipld {
    type Error = IpldTypeError;

    fn try_into(self) -> Result<IpldBool, Self::Error> {
        match self {
            Ipld::Bool(boolean) => Ok(boolean),
            _ => Err(IpldTypeError::NotABool),
        }
    }
}

#[derive(Debug)]
enum IpldTypeError {
    NotAString,
    NotABool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypedIpld<T> {
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
