//! `Ipld` error definitions.
use failure::Fail;
pub use failure::{format_err, Error};

/// Result alias.
pub type Result<T> = core::result::Result<T, Error>;

/// `Ipld` type error.
#[derive(Debug, Fail)]
pub enum IpldTypeError {
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
}

/// Not a key type error.
#[derive(Debug, Fail)]
#[fail(display = "Expected key type.")]
pub struct IpldKeyTypeError;
