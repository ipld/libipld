//! The `Ipld` crate.

#![deny(missing_docs)]
#![deny(warnings)]

pub mod codec;
pub mod dag;
pub mod error;
pub mod hash;
pub mod ipld;
pub mod macros;
pub mod path;
pub mod store;

pub use crate::codec::*;
pub use crate::dag::*;
pub use crate::error::*;
pub use crate::hash::*;
pub use crate::ipld::*;
pub use crate::path::*;
pub use crate::store::*;

/// Default prefix.
pub struct DefaultPrefix;

impl Prefix for DefaultPrefix {
    type Codec = DagCbor;
    type Hash = Blake2b;
}
