//! `Ipld` error definitions.

/// `Ipld` type error.
#[derive(Debug)]
pub enum IpldTypeError {
    /// Expected a `IpldString`.
    NotString,
    /// Expected a `IpldBool`.
    NotBool,
}
