//! `Ipld` error definitions.
use failure::Fail;

/// `Ipld` type error.
#[derive(Debug, Fail)]
pub enum IpldTypeError {
    /// Expected a `IpldNull`.
    #[fail(display = "Expected a `IpldNull`")]
    NotNull,
    /// Expected a `IpldBool`.
    #[fail(display = "Expected a `IpldBool`.")]
    NotBool,
    /// Expected a `IpldInteger`.
    #[fail(display = "Expected a `IpldInteger`.")]
    NotInteger,
    /// Expected a `IpldFloat`.
    #[fail(display = "Expected a `IpldFloat`.")]
    NotFloat,
    /// Expected a `IpldString`.
    #[fail(display = "Expected a `IpldString`.")]
    NotString,
    /// Expected a `IpldBytes`.
    #[fail(display = "Expected a `IpldBytes`.")]
    NotBytes,
    /// Expected a `IpldList`.
    #[fail(display = "Expected a `IpldList`.")]
    NotList,
    /// Expected a `IpldMap`.
    #[fail(display = "Expected a `IpldMap`.")]
    NotMap,
    /// Expected a `IpldLink`.
    #[fail(display = "Expected a `IpldLink`.")]
    NotLink,
}
