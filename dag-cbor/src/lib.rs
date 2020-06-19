//! CBOR codec.
#![deny(missing_docs)]
#![deny(warnings)]

use libipld_core::codec::{Codec, Decode, Encode};
use thiserror::Error;

pub mod decode;
pub mod encode;

/// CBOR codec.
#[derive(Clone, Copy, Debug)]
pub struct DagCborCodec;

impl Codec for DagCborCodec {
    type Error = Error;
}

/// Marker trait for types supporting the `DagCborCodec`.
pub trait DagCbor: Encode<DagCborCodec> + Decode<DagCborCodec> + decode::TryReadCbor {}

impl<T: Encode<DagCborCodec> + Decode<DagCborCodec> + decode::TryReadCbor> DagCbor for T {}

/// CBOR error.
#[derive(Debug, Error)]
pub enum Error {
    /// Number larger than u64.
    #[error("Number larger than u64.")]
    NumberOutOfRange,
    /// Length larger than usize or too small, for example zero length cid field.
    #[error("Length out of range.")]
    LengthOutOfRange,
    /// Unexpected cbor code.
    #[error("Unexpected cbor code.")]
    UnexpectedCode,
    /// Unknown cbor tag.
    #[error("Unkown cbor tag.")]
    UnknownTag,
    /// Unexpected key.
    #[error("Wrong key.")]
    UnexpectedKey,
    /// Unexpected eof.
    #[error("Unexpected end of file.")]
    UnexpectedEof,
    /// Io error.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// Utf8 error.
    #[error("{0}")]
    Utf8(#[from] std::str::Utf8Error),
    /// The byte before Cid was not multibase identity prefix.
    #[error("Invalid Cid prefix: {0}")]
    InvalidCidPrefix(u8),
    /// Cid error.
    #[error("{0}")]
    Cid(#[from] libipld_core::cid::Error),
    /// Ipld type error.
    #[error("{0}")]
    TypeError(#[from] libipld_core::error::TypeError),
}

/// CBOR result.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::CidGeneric;
    use libipld_core::multihash::Sha2_256;
    use libipld_macro::ipld;

    type Cid<H> = CidGeneric<u64, H>;

    #[test]
    fn test_encode_decode_cbor() {
        let cid = Cid::new_v1(0, Sha2_256::digest(b"cid"));
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": cid,
        });
        let bytes = DagCborCodec::encode(&ipld).unwrap();
        let ipld2 = DagCborCodec::decode(&bytes).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
