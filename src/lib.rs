//! The `Ipld` crate.

#![deny(missing_docs)]
#![deny(warnings)]

pub mod block;
pub mod codec;
pub mod convert;
pub mod dag;
pub mod error;
pub mod gc;
pub mod hash;
pub mod ipld;
pub mod macros;
pub mod path;
pub mod store;

pub use crate::block::*;
pub use crate::codec::*;
pub use crate::convert::*;
pub use crate::dag::*;
pub use crate::error::*;
pub use crate::gc::*;
pub use crate::hash::*;
pub use crate::ipld::*;
pub use crate::path::*;
pub use crate::store::*;

/// Default hash used.
pub type DefaultHash = Blake2b;

/// The maximum block size is 1MiB.
pub const MAX_BLOCK_SIZE: usize = 1_048_576;
