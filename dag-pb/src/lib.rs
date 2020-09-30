//! Protobuf codec.
#![deny(missing_docs)]
#![deny(warnings)]

pub use crate::codec::{PbLink, PbNode};
use core::convert::{TryFrom, TryInto};
use libipld_core::codec::{Codec, Decode, Encode};
use libipld_core::error::{Result, UnsupportedCodec};
use libipld_core::ipld::{Ipld, Size};
use std::io::{Read, Write};

mod codec;

/// Protobuf codec.
#[derive(Clone, Copy, Debug)]
pub struct DagPbCodec;

impl Codec for DagPbCodec {}

impl From<DagPbCodec> for u64 {
    fn from(_: DagPbCodec) -> Self {
        0x70
    }
}

impl TryFrom<u64> for DagPbCodec {
    type Error = UnsupportedCodec;

    fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<S: Size> Encode<DagPbCodec> for Ipld<S> {
    fn encode<W: Write>(&self, _: DagPbCodec, w: &mut W) -> Result<()> {
        let pb_node: PbNode<S> = self.try_into()?;
        let bytes = pb_node.into_bytes();
        w.write_all(&bytes)?;
        Ok(())
    }
}

impl<S: Size> Decode<DagPbCodec> for Ipld<S> {
    fn decode<R: Read>(_: DagPbCodec, r: &mut R) -> Result<Self> {
        let mut bytes = Vec::new();
        r.read_to_end(&mut bytes)?;
        Ok(PbNode::from_bytes(&bytes)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::multihash::{Code, MultihashCode};
    use libipld_core::raw::RawCodec;
    use libipld_core::Ipld;
    use std::collections::BTreeMap;

    #[test]
    fn test_encode_decode() {
        let digest = Code::Sha2_256.digest(&b"cid"[..]);
        let cid = Cid::new_v1(RawCodec.into(), digest);
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
