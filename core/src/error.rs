//! `Ipld` error definitions.
use crate::ipld::{Ipld, IpldIndex};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Expected {expected:?} but found {found:?}")]
pub struct TypeError {
    pub expected: TypeErrorType,
    pub found: TypeErrorType,
}

impl TypeError {
    pub fn new<A: Into<TypeErrorType>, B: Into<TypeErrorType>>(expected: A, found: B) -> Self {
        Self {
            expected: expected.into(),
            found: found.into(),
        }
    }
}

/// Type error type.
#[derive(Debug)]
pub enum TypeErrorType {
    /// Null type.
    Null,
    /// Boolean type.
    Bool,
    /// Integer type.
    Integer,
    /// Float type.
    Float,
    /// String type.
    String,
    /// Bytes type.
    Bytes,
    /// List type.
    List,
    /// Map type.
    Map,
    /// Link type.
    Link,
    /// Key type.
    Key(String),
    /// Index type.
    Index(usize),
}

impl From<&Ipld> for TypeErrorType {
    fn from(ipld: &Ipld) -> Self {
        match ipld {
            Ipld::Null => Self::Null,
            Ipld::Bool(_) => Self::Bool,
            Ipld::Integer(_) => Self::Integer,
            Ipld::Float(_) => Self::Float,
            Ipld::String(_) => Self::String,
            Ipld::Bytes(_) => Self::Bytes,
            Ipld::List(_) => Self::List,
            Ipld::Map(_) => Self::Map,
            Ipld::Link(_) => Self::Link,
        }
    }
}

impl From<IpldIndex<'_>> for TypeErrorType {
    fn from(index: IpldIndex<'_>) -> Self {
        match index {
            IpldIndex::List(i) => Self::Index(i),
            IpldIndex::Map(s) => Self::Key(s),
            IpldIndex::MapRef(s) => Self::Key(s.into()),
        }
    }
}
