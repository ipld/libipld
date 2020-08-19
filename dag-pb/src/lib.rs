//! Protobuf codec.
#![deny(missing_docs)]
#![deny(warnings)]

pub use crate::codec::{PbLink, PbNode};
use core::convert::TryInto;
use libipld_core::codec::{Codec, Decode, Encode};
use libipld_core::ipld::Ipld;
use std::io::{Read, Write};
use thiserror::Error;

mod codec;

/// Protobuf codec.
#[derive(Clone, Copy, Debug)]
pub struct DagPbCodec;

impl Codec for DagPbCodec {
    type Error = Error;
}

/// Protobuf error.
#[derive(Debug, Error)]
pub enum Error {
    /// Prost error.
    #[error(transparent)]
    Prost(#[from] prost::DecodeError),
    /// CID error.
    #[error(transparent)]
    Cid(#[from] libipld_core::cid::Error),
    /// Type error.
    #[error(transparent)]
    TypeError(#[from] libipld_core::error::TypeError),
    /// Io error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Encode<DagPbCodec> for Ipld {
    fn encode<W: Write>(&self, _: DagPbCodec, w: &mut W) -> Result<(), Error> {
        let pb_node: PbNode = self.try_into()?;
        let bytes = pb_node.into_bytes();
        w.write_all(&bytes)?;
        Ok(())
    }
}

impl Decode<DagPbCodec> for Ipld {
    fn decode<R: Read>(_: DagPbCodec, r: &mut R) -> Result<Self, Error> {
        let mut bytes = Vec::new();
        r.read_to_end(&mut bytes)?;
        Ok(PbNode::from_bytes(&bytes)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::{Cid, RAW};
    use libipld_core::multihash::{Multihash, MultihashDigest, SHA2_256};
    use std::collections::BTreeMap;

    #[test]
    fn test_encode_decode() {
        let digest = Multihash::new(SHA2_256, &b"cid"[..]).unwrap();
        let cid = Cid::new_v1(RAW, digest.to_raw().unwrap());
        let mut pb_link = BTreeMap::<String, Ipld>::new();
        pb_link.insert("Hash".to_string(), cid.into());
        pb_link.insert("Name".to_string(), "block".to_string().into());
        pb_link.insert("Tsize".to_string(), 13.into());

        let links: Vec<Ipld> = vec![pb_link.into()];
        let mut pb_node = BTreeMap::<String, Ipld>::new();
        pb_node.insert("Data".to_string(), b"Here is some data\n".to_vec().into());
        pb_node.insert("Links".to_string(), links.into());
        let data: Ipld = pb_node.into();

        let bytes = DagPbCodec.encode(&data).unwrap();
        let data2 = DagPbCodec.decode(&bytes).unwrap();
        assert_eq!(data, data2);
    }
}
