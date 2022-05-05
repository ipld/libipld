//! The `Ipld` crate.
#![deny(missing_docs)]
#![deny(warnings)]

pub mod block;
pub mod cache;
pub mod codec_impl;
pub mod mem;
pub mod path;
pub mod prelude;
pub mod store;

#[cfg(feature = "dag-cbor")]
pub use lurk_ipld_cbor as cbor;
#[cfg(all(feature = "dag-cbor", feature = "derive"))]
pub use lurk_ipld_cbor_derive::DagCbor;
pub use lurk_ipld_core::*;
#[cfg(feature = "dag-json")]
pub use lurk_ipld_json as json;
pub use lurk_ipld_macro::*;
#[cfg(feature = "dag-pb")]
pub use lurk_ipld_pb as pb;

pub use block::Block;
pub use cid::Cid;
pub use codec_impl::IpldCodec;
pub use error::Result;
pub use ipld::Ipld;
pub use link::Link;
pub use multihash::Multihash;
pub use path::{DagPath, Path};
pub use store::DefaultParams;
