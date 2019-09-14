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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::{Hash, Sha2_256};
    use crate::ipld;

    #[test]
    fn ipld_bool_from() {
        assert_eq!(Ipld::Bool(true), Ipld::from(true));
        assert_eq!(Ipld::Bool(false), Ipld::from(false));
    }

    #[test]
    fn ipld_integer_from() {
        assert_eq!(Ipld::Integer(1), Ipld::from(1i8));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i16));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i32));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i64));
        assert_eq!(Ipld::Integer(1), Ipld::from(1i128));

        //assert_eq!(Ipld::Integer(1), 1u8.to_ipld().to_owned());
        assert_eq!(Ipld::Integer(1), Ipld::from(1u16));
        assert_eq!(Ipld::Integer(1), Ipld::from(1u32));
        assert_eq!(Ipld::Integer(1), Ipld::from(1u64));
    }

    #[test]
    fn ipld_float_from() {
        assert_eq!(Ipld::Float(1.0), Ipld::from(1.0f32));
        assert_eq!(Ipld::Float(1.0), Ipld::from(1.0f64));
    }

    #[test]
    fn ipld_string_from() {
        assert_eq!(Ipld::String("a string".into()), Ipld::from("a string"));
        assert_eq!(
            Ipld::String("a string".into()),
            Ipld::from("a string".to_string())
        );
    }

    #[test]
    fn ipld_bytes_from() {
        assert_eq!(
            Ipld::Bytes(vec![0, 1, 2, 3]),
            Ipld::from(&[0u8, 1u8, 2u8, 3u8][..])
        );
        assert_eq!(
            Ipld::Bytes(vec![0, 1, 2, 3]),
            Ipld::from(vec![0u8, 1u8, 2u8, 3u8])
        );
    }

    #[test]
    fn ipld_link_from() {
        let data = vec![0, 1, 2, 3];
        let hash = Sha2_256::digest(&data);
        let cid = Cid::new_v0(hash).unwrap();
        assert_eq!(Ipld::Link(cid.clone()), Ipld::from(cid));
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
