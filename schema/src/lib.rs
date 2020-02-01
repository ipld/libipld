//! IPLD Schemas and Representations

extern crate derive_more;
#[macro_use]
extern crate lazy_static;

mod codec;
mod error;
mod ipld;
//mod link;
mod representation;

// public internal and dependency exports
pub use crate::{
    codec::DagCbor,
    error::Error,
    ipld::BorrowedIpld,
    //    link::Link,
    representation::{context::Context, Representation},
};

/// External imports, re-exported for convenience and for `libipld-schema-derive`
pub mod dev {
    pub use async_trait::async_trait;
    pub use cid::{self, Cid};
    pub use libipld_base::{
        codec::{Codec, CodecExt, IpldVisitor},
        ipld::{Ipld, IpldIndex},
    };
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[cfg(feature = "derive")]
    #[macro_use]
    pub use libipld_schema_derive::schema;
}
