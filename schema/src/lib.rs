#![feature(specialization)]

mod link;
mod representation;
mod schema;

// public internal and dependency exports
pub use crate::{
    link::Link,
    representation::{
        error::Error, BlockReadContext, BlockWriteContext, Mutable, Queryable, ReadContext,
        RecursiveContext, Representation, WriteContext,
    },
};
pub use async_trait::async_trait;
pub use libipld::{
    cbor::{
        decode::{self, Read},
        encode::{self, Write},
    },
    cid::Cid,
    ipld::{Ipld, IpldIndex},
};

// internal exports for convenience
pub(crate) use libipld::{
    cbor::{CborError, ReadCbor, WriteCbor},
    cid::Error as CidError,
    error::{BlockError, IpldError},
};
pub(crate) use std::collections::BTreeMap;
