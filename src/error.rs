//! `Ipld` error definitions.
use failure::Fail;

/// `Ipld` type error.
#[derive(Debug, Fail)]
pub enum IpldTypeError {
    /// Expected a `IpldString`.
    #[fail(display = "Expected a `String`.")]
    NotString,
    /// Expected a `IpldBool`.
    #[fail(display = "Expected a `bool`.")]
    NotBool,
}
