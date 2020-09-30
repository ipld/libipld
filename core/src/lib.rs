//! Core ipld types used by ipld codecs.
#![deny(missing_docs)]
#![deny(warnings)]

pub mod codec;
pub mod convert;
pub mod error;
pub mod ipld;
pub mod raw;

pub use cid;
pub use cid::multibase;
pub use cid::multihash;

/// IPLD with a default allocated size for CIDs/Multihashs
pub type Ipld = ipld::Ipld<multihash::U64>;
