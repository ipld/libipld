//! Core ipld types used by ipld codecs.
#![deny(missing_docs)]
#![deny(warnings)]

pub mod codec;
pub mod convert;
pub mod error;
pub mod ipld;
pub mod raw;

pub use multibase;
pub use tiny_cid as cid;
pub use tiny_multihash as multihash;

/// The maximum block size is 1MiB.
pub const MAX_BLOCK_SIZE: usize = 1_048_576;
