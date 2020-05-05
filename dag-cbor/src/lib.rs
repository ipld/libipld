//! CBOR codec.
use libipld_core::cid;
pub use libipld_core::codec::Codec;
use libipld_core::error::BlockError;
pub use libipld_core::error::IpldError;
use libipld_core::ipld::Ipld;
use thiserror::Error;

pub mod decode;
pub mod encode;

pub use decode::ReadCbor;
pub use encode::WriteCbor;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCborCodec;

impl DagCborCodec {
    pub const CODEC: cid::Codec = cid::Codec::DagCBOR;

    pub fn encode(ipld: &Ipld) -> Result<Box<[u8]>, CborError> {
        let mut bytes = Vec::new();
        ipld.write_cbor(&mut bytes)?;
        Ok(bytes.into_boxed_slice())
    }

    pub fn decode(mut data: &[u8]) -> Result<Ipld, CborError> {
        Ipld::read_cbor(&mut data)
    }
}

impl Codec for DagCborCodec {
    fn codec(&self) -> cid::Codec {
        Self::CODEC
    }
    fn encode(&self, ipld: &Ipld) -> Result<Box<[u8]>, BlockError> {
        Self::encode(ipld).map_err(|err| err.into())
    }

    fn decode(&self, data: &[u8]) -> Result<Ipld, BlockError> {
        Self::decode(data).map_err(|err| err.into())
    }
}

/// CBOR error.
#[derive(Debug, Error)]
pub enum CborError {
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
    Cid(#[from] cid::Error),
    /// Ipld error.
    #[error("{0}")]
    Ipld(#[from] IpldError),
}

impl From<CborError> for BlockError {
    fn from(err: CborError) -> Self {
        Self::CodecError(err.into())
    }
}

/// CBOR result.
pub type CborResult<T> = Result<T, CborError>;

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::multihash::Sha2_256;
    use libipld_macro::ipld;

    #[test]
    fn test_encode_decode_cbor() {
        let cid = Cid::new_v0(Sha2_256::digest(b"cid")).unwrap();
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
