//! `Ipld` error definitions.
use failure::Fail;
pub use failure::{format_err, Error};

/// Result alias.
pub type Result<T> = core::result::Result<T, Error>;

/// `Ipld` type error.
#[derive(Debug, Fail)]
pub enum IpldError {
    /// Expected a boolean.
    #[fail(display = "Expected a boolean.")]
    NotBool,
    /// Expected an integer.
    #[fail(display = "Expected an integer.")]
    NotInteger,
    /// Expected a float.
    #[fail(display = "Expected a float.")]
    NotFloat,
    /// Expected a string.
    #[fail(display = "Expected a string.")]
    NotString,
    /// Expected bytes.
    #[fail(display = "Expected bytes.")]
    NotBytes,
    /// Expected a list.
    #[fail(display = "Expected a list.")]
    NotList,
    /// Expected a map.
    #[fail(display = "Expected a map.")]
    NotMap,
    /// Expected a cid.
    #[fail(display = "Expected a cid.")]
    NotLink,
    /// Expected a key.
    #[fail(display = "Expected a key.")]
    NotKey,
    /// Index not found.
    #[fail(display = "Index not found.")]
    IndexNotFound,
    /// Key not found.
    #[fail(display = "Key not found.")]
    KeyNotFound,
    /// Other.
    #[fail(display = "{}", _0)]
    Other(Error),
}

impl From<Error> for IpldError {
    fn from(err: Error) -> Self {
        IpldError::Other(err)
    }
}

impl From<core::convert::Infallible> for IpldError {
    fn from(_: core::convert::Infallible) -> Self {
        unreachable!();
    }
}

/// Block error.
#[derive(Debug, Fail)]
pub enum BlockError {
    /// Block exceeds MAX_BLOCK_SIZE.
    #[fail(display = "Block size {} exceeds MAX_BLOCK_SIZE.", _0)]
    BlockToLarge(usize),
    /// Hash does not match the CID.
    #[fail(display = "Hash does not match the CID.")]
    InvalidHash,
    /// The codec is unsupported.
    #[fail(display = "Unsupported codec {:?}.", _0)]
    UnsupportedCodec(cid::Codec),
    /// The codec returned an error.
    #[fail(display = "Codec error: {}", _0)]
    CodecError(Error),
}
