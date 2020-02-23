//! `Ipld` error definitions.
use failure::{format_err, Fail};
use multihash::Multihash;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};

/// Result alias.
pub type Result<T> = core::result::Result<T, BlockError>;

/// Ipld type error.
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
    InvalidHash(Multihash),
    /// The codec is unsupported.
    #[fail(display = "Unsupported codec {:?}.", _0)]
    UnsupportedCodec(cid::Codec),
    /// The codec returned an error.
    #[fail(display = "Codec error: {}", _0)]
    CodecError(failure::Error),
    /// Io error.
    #[fail(display = "{}", _0)]
    Io(std::io::Error),
    // RwLock error
    #[fail(display = "{}", _0)]
    RwLockReadGuard(failure::Error),
    #[fail(display = "{}", _0)]
    PoisonError(failure::Error),
    #[fail(display = "{}", _0)]
    RwLockWriteGuard(failure::Error),
    /// Cid error.
    #[fail(display = "{}", _0)]
    Cid(cid::Error),
    /// Link error.
    #[fail(display = "Invalid link.")]
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

impl<T: std::fmt::Debug> From<RwLockReadGuard<'_, T>> for BlockError {
    fn from(err: RwLockReadGuard<'_, T>) -> Self {
        Self::RwLockReadGuard(format_err!("{:?}", err))
    }
}

impl<T: std::fmt::Debug> From<PoisonError<T>> for BlockError {
    fn from(err: PoisonError<T>) -> Self {
        Self::PoisonError(format_err!("{:?}", err))
    }
}

impl<T: std::fmt::Debug> From<RwLockWriteGuard<'_, T>> for BlockError {
    fn from(err: RwLockWriteGuard<T>) -> Self {
        Self::RwLockWriteGuard(format_err!("{:?}", err))
    }
}
