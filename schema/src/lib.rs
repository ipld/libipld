//! IPLD Schemas and Representations

extern crate derive_more;

mod error;
//mod link;
mod representation;

// public internal and dependency exports
pub use crate::{
    error::Error,
    //    link::Link,
    representation::{
        context::{
            self,
            commands::{self, Command, ResolveBlock, ResolveRange},
            Context, Handler, IntoHandler,
        },
        Representation,
    },
};

/// External imports, re-exported for convenience and for `libipld-schema-derive`
pub mod dev {
    pub use async_std::io::{
        prelude::{ReadExt, SeekExt, WriteExt},
        Read, Seek, SeekFrom, Write,
    };
    pub use async_trait::async_trait;
    pub use bytes::Bytes;
    pub use cid::Cid;
    pub use dag_cbor::{CborError, ReadCbor, WriteCbor};
    pub use libipld_base::{
        codec::{Codec, CodecExt},
        ipld::IpldIndex,
    };

    #[cfg(feature = "derive")]
    #[macro_use]
    pub use libipld_schema_derive::schema;
}
