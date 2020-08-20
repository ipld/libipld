//! `Ipld` error definitions.
use crate::ipld::{Ipld, IpldIndex};
pub use anyhow::{Error, Result};
use thiserror::Error;

/// Block exceeds 1MiB.
#[derive(Debug, Error)]
#[error("Block size {0} exceeds 1MiB.")]
pub struct BlockTooLarge(pub usize);

/// The codec is unsupported.
#[derive(Debug, Error)]
#[error("Unsupported codec {0:?}.")]
pub struct UnsupportedCodec(pub u64);

/// The multihash is unsupported.
#[derive(Debug, Error)]
#[error("Unsupported multihash {0:?}.")]
pub struct UnsupportedMultihash(pub u64);

/// Hash does not match the CID.
#[derive(Debug, Error)]
#[error("Hash of data does not match the CID.")]
pub struct InvalidMultihash(pub Vec<u8>);

/// The block wasn't found. The supplied string is a CID.
#[derive(Debug, Error)]
#[error("Failed to retrive block {0}.")]
pub struct BlockNotFound(pub String);

/// The batch was empty.
#[derive(Debug, Error)]
#[error("Tried to insert an empty batch.")]
pub struct EmptyBatch;

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
