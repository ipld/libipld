//! `Ipld` codecs.

mod canon;
mod dag_cbor;
mod borrowed;

pub use dag_cbor::DagCbor;
pub use borrowed::{Ipld, IpldListIter, IpldMapIter};
