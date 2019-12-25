//! IPLD Schemas and Representations
#![feature(specialization)]

extern crate derive_more;

mod error;
mod link;
mod representation;

#[macro_use]
mod schema;
#[macro_use]
mod advanced;

// public internal and dependency exports
pub use crate::{
    error::Error,
    link::Link,
    representation::{
        context::{self, Context},
        Representation,
    },
};
pub use bytes::Bytes;
pub use libipld::{
    cbor::{decode::Read, encode::Write},
    cid::Cid,
    ipld::{Ipld, IpldIndex},
};

// internal exports for convenience
pub(crate) use async_trait::async_trait;
pub(crate) use libipld::{
    cbor::{encode, CborError, ReadCbor, WriteCbor},
    cid::Error as CidError,
    error::{BlockError, IpldError},
};
pub(crate) use std::collections::BTreeMap;
