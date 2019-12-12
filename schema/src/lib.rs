#![feature(generic_associated_types)]

mod advanced;
mod link;
pub mod schema;

pub use async_trait::async_trait;
pub use libipld::*;
pub use link::Link;
pub use std::collections::BTreeMap;
