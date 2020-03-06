//! CBOR codec.
use failure::Fail;
pub use libipld_base::codec::Codec;
use libipld_base::error::BlockError;
pub use libipld_base::error::IpldError;
use libipld_base::ipld::Ipld;

pub mod decode;
pub mod encode;

pub use decode::ReadCbor;
pub use encode::WriteCbor;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCborCodec;

impl Codec for DagCborCodec {
    const VERSION: libipld_base::cid::Version = libipld_base::cid::Version::V1;
    const CODEC: libipld_base::cid::Codec = libipld_base::cid::Codec::DagCBOR;

    type Error = CborError;

    fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error> {
        let mut bytes = Vec::new();
        ipld.write_cbor(&mut bytes)?;
        Ok(bytes.into_boxed_slice())
    }

    fn decode(mut data: &[u8]) -> Result<Ipld, Self::Error> {
        Ipld::read_cbor(&mut data)
    }
}

/// CBOR error.
#[derive(Debug, Fail)]
pub enum CborError {
    /// Number larger than u64.
    #[fail(display = "Number larger than u64.")]
    NumberOutOfRange,
    /// Length larger than usize.
    #[fail(display = "Length out of range.")]
    LengthOutOfRange,
    /// Unexpected cbor code.
    #[fail(display = "Unexpected cbor code.")]
    UnexpectedCode,
    /// Unknown cbor tag.
    #[fail(display = "Unkown cbor tag.")]
    UnknownTag,
    /// Unexpected key.
    #[fail(display = "Wrong key.")]
    UnexpectedKey,
    /// Unexpected eof.
    #[fail(display = "Unexpected end of file.")]
    UnexpectedEof,
    /// Io error.
    #[fail(display = "{}", _0)]
    Io(std::io::Error),
    /// Utf8 error.
    #[fail(display = "{}", _0)]
    Utf8(std::str::Utf8Error),
    /// Cid error.
    #[fail(display = "{}", _0)]
    Cid(libipld_base::cid::Error),
    /// Ipld error.
    #[fail(display = "{}", _0)]
    Ipld(libipld_base::error::IpldError),
}

impl From<std::io::Error> for CborError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            _ => Self::Io(err),
        }
    }
}

impl From<std::str::Utf8Error> for CborError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<libipld_base::cid::Error> for CborError {
    fn from(err: libipld_base::cid::Error) -> Self {
        Self::Cid(err)
    }
}

impl From<libipld_base::error::IpldError> for CborError {
    fn from(err: libipld_base::error::IpldError) -> Self {
        Self::Ipld(err)
    }
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
    use libipld_base::cid::Cid;
    use libipld_macro::ipld;

    #[test]
    fn test_encode_decode_cbor() {
        let cid = Cid::new_v0(multihash::Sha2_256::digest(b"cid")).unwrap();
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
