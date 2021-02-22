//! CBOR codec.
#![deny(missing_docs)]
#![deny(warnings)]

use core::convert::TryFrom;
use libipld_core::codec::{Codec, Decode, Encode};
pub use libipld_core::error::{Result, UnsupportedCodec};

pub mod decode;
pub mod encode;
pub mod error;

/// CBOR codec.
#[derive(Clone, Copy, Debug, Default)]
pub struct DagCborCodec;

impl Codec for DagCborCodec {}

impl From<DagCborCodec> for u64 {
    fn from(_: DagCborCodec) -> Self {
        0x71
    }
}

impl TryFrom<u64> for DagCborCodec {
    type Error = UnsupportedCodec;

    fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}

/// Marker trait for types supporting the `DagCborCodec`.
pub trait DagCbor: Encode<DagCborCodec> + Decode<DagCborCodec> {}

impl<T: Encode<DagCborCodec> + Decode<DagCborCodec>> DagCbor for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::ipld::Ipld;
    use libipld_core::multihash::{Code, MultihashDigest};
    use libipld_macro::ipld;
    use std::collections::HashSet;

    #[test]
    fn test_encode_decode_cbor() {
        let cid = Cid::new_v1(0, Code::Blake3_256.digest(&b"cid"[..]));
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": cid,
        });
        let bytes = DagCborCodec.encode(&ipld).unwrap();
        let ipld2 = DagCborCodec.decode(&bytes).unwrap();
        assert_eq!(ipld, ipld2);
    }

    #[test]
    fn test_references() {
        let cid = Cid::new_v1(0, Code::Blake3_256.digest(&b"0"[..]));
        let ipld = ipld!({
            "list": [true, cid],
        });
        let bytes = DagCborCodec.encode(&ipld).unwrap();
        let mut set = HashSet::new();
        DagCborCodec
            .references::<Ipld, _>(&bytes, &mut set)
            .unwrap();
        assert!(set.contains(&cid));
    }
}
