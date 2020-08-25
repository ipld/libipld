//! CBOR error types.
use thiserror::Error;

/// Number larger than u64.
#[derive(Debug, Error)]
#[error("Number larger than u64.")]
pub struct NumberOutOfRange;

/// Length larger than usize or too small, for example zero length cid field.
#[derive(Debug, Error)]
#[error("Length out of range.")]
pub struct LengthOutOfRange;

/// Unexpected cbor code.
#[derive(Debug, Error)]
#[error("Unexpected cbor code.")]
pub struct UnexpectedCode;

/// Unknown cbor tag.
#[derive(Debug, Error)]
#[error("Unkown cbor tag.")]
pub struct UnknownTag;

/// Unexpected key.
#[derive(Debug, Error)]
#[error("Wrong key.")]
pub struct UnexpectedKey;

/// Unexpected eof.
#[derive(Debug, Error)]
#[error("Unexpected end of file.")]
pub struct UnexpectedEof;

/// The byte before Cid was not multibase identity prefix.
#[derive(Debug, Error)]
#[error("Invalid Cid prefix: {0}")]
pub struct InvalidCidPrefix(pub u8);
