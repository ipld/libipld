//! `Ipld` error definitions.
use failure::Fail;
use multihash::Multihash;

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
    /// Unexpected end of file.
    #[fail(display = "Unexpected end of file.")]
    UnexpectedEof,
    /// The codec returned an error.
    #[fail(display = "Codec error: {}", _0)]
    CodecError(failure::Error),
    /// Io error.
    #[fail(display = "{}", _0)]
    Io(std::io::Error),
}

impl From<std::io::Error> for BlockError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Dag error.
#[derive(Debug, Fail)]
pub enum DagError {
    /// Path segment is not a number.
    #[fail(display = "Path segment is not a number.")]
    NotNumber(std::num::ParseIntError),
    /// Cannot index into ipld.
    #[fail(display = "Cannot index into")]
    NotIndexable,
    /// Ipld error.
    #[fail(display = "{}", _0)]
    Ipld(IpldError),
    /// Block error.
    #[fail(display = "{}", _0)]
    Block(BlockError),
}

impl From<std::num::ParseIntError> for DagError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::NotNumber(err)
    }
}

impl From<IpldError> for DagError {
    fn from(err: IpldError) -> Self {
        Self::Ipld(err)
    }
}

impl From<BlockError> for DagError {
    fn from(err: BlockError) -> Self {
        Self::Block(err)
    }
}
