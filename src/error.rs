//! `Ipld` error definitions.
use crate::cid::Cid;
use crate::multihash::Multihash;
pub use libipld_core::error::*;
use thiserror::Error;

/// Result alias.
pub type Result<T> = core::result::Result<T, Error>;

/// Ipld error.
#[derive(Debug, Error)]
pub enum Error {
    /// Block exceeds MAX_BLOCK_SIZE.
    #[error("Block size {0} exceeds MAX_BLOCK_SIZE.")]
    BlockTooLarge(usize),
    /// Hash does not match the CID.
    #[error("Hash does not match the CID.")]
    InvalidHash(Multihash),
    /// The codec is unsupported.
    #[error("Unsupported codec {0:?}.")]
    UnsupportedCodec(crate::codec::Code),
    /// The multihash is unsupported.
    #[error("Unsupported multihash {0:?}.")]
    UnsupportedMultihash(crate::multihash::Code),
    /// Type error.
    #[error("{0}")]
    TypeError(#[from] TypeError),
    /// The codec returned an error.
    #[error("Codec error: {0}")]
    CodecError(Box<dyn std::error::Error + Send>),
    /// The store returned an error.
    #[error("{0}")]
    StoreError(#[from] StoreError),
}

/// Store error.
#[derive(Debug, Error)]
pub enum StoreError {
    /// The block wasn't found.
    #[error("failed to retrive block {0}")]
    BlockNotFound(Cid),
    /// Io operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
