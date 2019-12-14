#![feature(associated_type_defaults)]
#![feature(specialization)]

mod advanced;
mod link;
mod representation;
mod schema;

pub use async_trait::async_trait;
pub use libipld::{
    cbor::{decode::Read, encode::Write, CborError, ReadCbor, WriteCbor},
    cid::Cid,
    error::BlockError,
    ipld::IpldIndex,
};
pub use link::Link;
pub use representation::{
    context::{BlockContext, Context, RecursiveContext},
    Representation,
};
pub use std::collections::BTreeMap;
