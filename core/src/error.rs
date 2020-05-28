//! `Ipld` error definitions.
use crate::ipld::{Ipld, IpldIndex};
use std::convert::TryFrom;
use thiserror::Error;

/// Type error.
#[derive(Debug, Error)]
#[error("Expected {expected:?} but found {found:?}")]
pub struct TypeError {
    /// The expected type.
    pub expected: TypeErrorType,
    /// The actual type.
    pub found: TypeErrorType,
}

impl TypeError {
    /// Creates a new type error.
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

impl<C, H> From<&Ipld<C, H>> for TypeErrorType
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn from(ipld: &Ipld<C, H>) -> Self {
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
