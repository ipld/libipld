//! `Ipld` types.
//!
//! Every `Ipld` type is required to implement `From` and `Into` for all
//! relevant Rust types.
//!
//! Every `Ipld` type implements `From<Ipld>` and `From<TypedIpld<T>>`.

/// Represents a `String` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldString(String);

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

impl From<&str> for IpldString {
    fn from(string: &str) -> IpldString {
        IpldString(string.to_string())
    }
}

/// Represents a `bool` in `Ipld`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpldBool(bool);

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
