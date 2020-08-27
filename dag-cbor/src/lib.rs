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
#[derive(Clone, Copy, Debug)]
pub struct DagCborCodec;

impl Codec for DagCborCodec {}

impl From<DagCborCodec> for u64 {
    fn from(_: DagCborCodec) -> Self {
        libipld_core::cid::DAG_CBOR
    }
}

impl TryFrom<u64> for DagCborCodec {
    type Error = UnsupportedCodec;

    fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}

/// Marker trait for types supporting the `DagCborCodec`.
pub trait DagCbor: Encode<DagCborCodec> + Decode<DagCborCodec> + decode::TryReadCbor {}

impl<T: Encode<DagCborCodec> + Decode<DagCborCodec> + decode::TryReadCbor> DagCbor for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::multihash::{Multihash, MultihashDigest, SHA2_256};
    use libipld_macro::ipld;

    #[test]
    fn test_encode_decode_cbor() {
        let cid = Cid::new_v1(
            0,
            Multihash::new(SHA2_256, &b"cid"[..])
                .unwrap()
                .to_raw()
                .unwrap(),
        );
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
}
