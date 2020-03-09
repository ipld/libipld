//! `Ipld` error definitions.
use multihash::Multihash;
use thiserror::Error;

/// Result alias.
pub type Result<T> = core::result::Result<T, BlockError>;

/// Ipld type error.
#[derive(Debug, Error)]
pub enum IpldError {
    /// Expected a boolean.
    #[error("Expected a boolean.")]
    NotBool,
    /// Expected an integer.
    #[error("Expected an integer.")]
    NotInteger,
    /// Expected a float.
    #[error("Expected a float.")]
    NotFloat,
    /// Expected a string.
    #[error("Expected a string.")]
    NotString,
    /// Expected bytes.
    #[error("Expected bytes.")]
    NotBytes,
    /// Expected a list.
    #[error("Expected a list.")]
    NotList,
    /// Expected a map.
    #[error("Expected a map.")]
    NotMap,
    /// Expected a cid.
    #[error("Expected a cid.")]
    NotLink,
    /// Expected a key.
    #[error("Expected a key.")]
    NotKey,
    /// Index not found.
    #[error("Index not found.")]
    IndexNotFound,
    /// Key not found.
    #[error("Key not found.")]
    KeyNotFound,
}

impl From<core::convert::Infallible> for IpldError {
    fn from(_: core::convert::Infallible) -> Self {
        unreachable!();
    }
}

/// Block error.
#[derive(Debug, Error)]
pub enum BlockError {
    /// Block exceeds MAX_BLOCK_SIZE.
    #[error("Block size {0} exceeds MAX_BLOCK_SIZE.")]
    BlockToLarge(usize),
    /// Hash does not match the CID.
    #[error("Hash does not match the CID.")]
    InvalidHash(Multihash),
    /// The codec is unsupported.
    #[error("Unsupported codec {0:?}.")]
    UnsupportedCodec(cid::Codec),
    /// The multihash is unsupported.
    #[error("Unsupported multihash {0:?}.")]
    UnsupportedMultihash(multihash::Code),
    /// The codec returned an error.
    #[error("Codec error: {0}")]
    CodecError(Box<dyn std::error::Error + Send + Sync>),
    /// Io error.
    #[error("{0}")]
    Io(std::io::Error),
    /// Cid error.
    #[error("{0}")]
    Cid(cid::Error),
    /// Link error.
    #[error("Invalid link.")]
    InvalidLink,
}

impl From<std::io::Error> for BlockError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<cid::Error> for BlockError {
    fn from(err: cid::Error) -> Self {
        Self::Cid(err)
    }
}
