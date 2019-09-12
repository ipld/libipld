//! Ipld representation.

pub use cid::Cid;
use std::collections::BTreeMap;

/// Ipld
#[derive(Clone, Debug, PartialEq)]
pub enum Ipld {
    /// Represents the absence of a value or the value undefined.
    Null,
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an integer.
    Integer(i128),
    /// Represents a floating point value.
    Float(f64),
    /// Represents an UTF-8 string.
    String(String),
    /// Represents a sequence of bytes.
    Bytes(Vec<u8>),
    /// Represents a list.
    List(Vec<Ipld>),
    /// Represents a map.
    Map(BTreeMap<String, Ipld>),
    /// Represents a link to an Ipld node
    Link(Cid),
}

/// Ipld ref
#[derive(Clone, Debug, PartialEq)]
pub enum IpldRef<'a> {
    /// Represents the absence of a value or the value undefined.
    Null,
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an integer.
    Integer(i128),
    /// Represents a floating point value.
    Float(f64),
    /// Represents an UTF-8 string.
    String(&'a str),
    /// Represents a sequence of bytes.
    Bytes(&'a [u8]),
    /// Represents a list.
    List(&'a [Ipld]),
    /// Represents an owned list.
    OwnedList(Vec<IpldRef<'a>>),
    /// Represents a map.
    Map(&'a BTreeMap<String, Ipld>),
    /// Represents an owned map.
    OwnedMap(BTreeMap<String, IpldRef<'a>>),
    /// Represents a link to an Ipld node
    Link(&'a Cid),
}

impl<'a> From<&'a Ipld> for IpldRef<'a> {
    fn from(ipld: &'a Ipld) -> Self {
        ipld.as_ref()
    }
}

impl<'a> From<IpldRef<'a>> for Ipld {
    fn from(ipld: IpldRef<'a>) -> Self {
        ipld.to_owned()
    }
}

/// An index into ipld
pub enum IpldIndex<'a> {
    /// An index into an ipld list.
    List(usize),
    /// An owned index into an ipld map.
    Map(String),
    /// An index into an ipld map.
    MapRef(&'a str),
}

impl<'a> From<usize> for IpldIndex<'a> {
    fn from(index: usize) -> Self {
        Self::List(index)
    }
}

impl<'a> From<String> for IpldIndex<'a> {
    fn from(key: String) -> Self {
        Self::Map(key)
    }
}

impl<'a> From<&'a str> for IpldIndex<'a> {
    fn from(key: &'a str) -> Self {
        Self::MapRef(key)
    }
}

impl Ipld {
    /// Indexes into a ipld list or map.
    pub fn get<'a, T: Into<IpldIndex<'a>>>(&self, index: T) -> Option<&Ipld> {
        match self {
            Ipld::List(l) => match index.into() {
                IpldIndex::List(i) => l.get(i),
                _ => None,
            },
            Ipld::Map(m) => match index.into() {
                IpldIndex::Map(ref key) => m.get(key),
                IpldIndex::MapRef(key) => m.get(key),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns a ipld reference.
    pub fn as_ref<'a>(&'a self) -> IpldRef<'a> {
        match self {
            Ipld::Null => IpldRef::Null,
            Ipld::Bool(b) => IpldRef::Bool(*b),
            Ipld::Integer(i) => IpldRef::Integer(*i),
            Ipld::Float(f) => IpldRef::Float(*f),
            Ipld::String(ref s) => IpldRef::String(s),
            Ipld::Bytes(ref b) => IpldRef::Bytes(b),
            Ipld::List(ref l) => IpldRef::List(l),
            Ipld::Map(ref m) => IpldRef::Map(m),
            Ipld::Link(ref c) => IpldRef::Link(c),
        }
    }
}

impl<'a> IpldRef<'a> {
    /// Turns an ipld reference into an owned ipld.
    pub fn to_owned(self) -> Ipld {
        match self {
            IpldRef::Null => Ipld::Null,
            IpldRef::Bool(b) => Ipld::Bool(b),
            IpldRef::Integer(i) => Ipld::Integer(i),
            IpldRef::Float(f) => Ipld::Float(f),
            IpldRef::String(s) => Ipld::String(s.to_string()),
            IpldRef::Bytes(b) => Ipld::Bytes(b.to_vec()),
            IpldRef::List(l) => Ipld::List(l.to_vec()),
            IpldRef::OwnedList(l) => Ipld::List(l.into_iter().map(|v| v.to_owned()).collect()),
            IpldRef::Map(m) => Ipld::Map((*m).clone()),
            IpldRef::OwnedMap(m) => {
                Ipld::Map(m.into_iter().map(|(k, v)| (k, v.to_owned())).collect())
            }
            IpldRef::Link(c) => Ipld::Link((*c).clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::ToIpld;
    use crate::hash::{Hash, Sha2_256};
    use crate::ipld;

    #[test]
    fn ipld_bool_from() {
        assert_eq!(Ipld::Bool(true), true.to_ipld().to_owned());
        assert_eq!(Ipld::Bool(false), false.to_ipld().to_owned());
    }

    #[test]
    fn ipld_integer_from() {
        assert_eq!(Ipld::Integer(1), 1i8.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1i16.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1i32.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1i64.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1i128.to_ipld().to_owned());

        //assert_eq!(Ipld::Integer(1), 1u8.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1u16.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1u32.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), 1u64.to_ipld().to_owned());
    }

    #[test]
    fn ipld_float_from() {
        assert_eq!(Ipld::Float(1.0), 1.0f32.to_ipld().to_owned());
        assert_eq!(Ipld::Float(1.0), 1.0f64.to_ipld().to_owned());
    }

    #[test]
    fn ipld_string_from() {
        assert_eq!(Ipld::String("a string".into()), "a string".to_ipld().to_owned());
        assert_eq!(
            Ipld::String("a string".into()),
            Ipld::from("a string".to_string().to_ipld().to_owned())
        );
    }

    #[test]
    fn ipld_bytes_from() {
        assert_eq!(Ipld::Bytes(vec![0, 1, 2, 3]), (&[0u8, 1u8, 2u8, 3u8][..]).to_ipld().to_owned());
        assert_eq!(Ipld::Bytes(vec![0, 1, 2, 3]), vec![0u8, 1u8, 2u8, 3u8].to_ipld().to_owned());
    }

    #[test]
    fn ipld_link_from() {
        let data = vec![0, 1, 2, 3];
        let hash = Sha2_256::digest(&data);
        let cid = Cid::new_v0(hash).unwrap();
        assert_eq!(Ipld::Link(cid.clone()), cid.to_ipld().to_owned());
    }

    #[test]
    fn index() {
        let ipld = ipld!([0, 1, 2]);
        assert_eq!(ipld.get(0).unwrap(), &Ipld::Integer(0));
        assert_eq!(ipld.get(1).unwrap(), &Ipld::Integer(1));
        assert_eq!(ipld.get(2).unwrap(), &Ipld::Integer(2));

        let ipld = ipld!({
            "a": 0,
            "b": 1,
            "c": 2,
        });
        assert_eq!(ipld.get("a").unwrap(), &Ipld::Integer(0));
    }
}
