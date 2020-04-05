//! The `Ipld` crate.

#![deny(missing_docs)]
#![deny(warnings)]

pub mod block;
pub mod dag;
pub mod gc;
pub mod path;
pub mod store;

pub use dag_cbor as cbor;
pub use dag_cbor_derive::DagCbor;
#[cfg(feature = "dag-json")]
pub use dag_json as json;
#[cfg(feature = "dag-pb")]
pub use dag_pb as pb;
pub use libipld_base::*;
pub use libipld_macro::*;

/// Default hash used.
pub type DefaultHash = hash::Blake2b256;

/// The maximum block size is 1MiB.
pub const MAX_BLOCK_SIZE: usize = 1_048_576;
