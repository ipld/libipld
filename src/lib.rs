//! The `Ipld` crate.

#![deny(missing_docs)]
#![deny(warnings)]

pub mod block;
pub mod error;
pub mod mem;
pub mod path;
pub mod store;

#[cfg(feature = "dag-cbor")]
pub use dag_cbor as cbor;
#[cfg(all(feature = "dag-cbor", feature = "derive"))]
pub use dag_cbor_derive::DagCbor;
#[cfg(feature = "dag-json")]
pub use dag_json as json;
#[cfg(feature = "dag-pb")]
pub use dag_pb as pb;
pub use libipld_core::*;
pub use libipld_macro::*;

/// The maximum block size is 1MiB.
pub const MAX_BLOCK_SIZE: usize = 1_048_576;
