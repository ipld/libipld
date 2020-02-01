//! `Ipld` codecs.

mod dag_cbor;

pub use dag_cbor::DagCbor;

use crate::{dev::*, Error};
use core::fmt::Debug;
use failure::Fail;

/// Codec trait.
pub trait Codec {
    /// Codec version.
    const VERSION: cid::Version;

    /// Codec code.
    const CODEC: cid::Codec;

    /// Error type.
    type Error: Debug + Fail + Into<Error>;


}
