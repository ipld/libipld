//! CBOR codec.
use crate::codec::Codec;
use crate::error::BlockError;
use crate::ipld::Ipld;
use async_trait::async_trait;
use failure::Fail;

pub mod decode;
pub mod encode;

pub use decode::ReadCbor;
pub use encode::WriteCbor;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCborCodec;

#[async_trait]
impl Codec for DagCborCodec {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    type Error = CborError;

    async fn encode(ipld: &Ipld) -> Result<Box<[u8]>, Self::Error> {
        let mut bytes = Vec::new();
        ipld.write_cbor(&mut bytes).await?;
        Ok(bytes.into_boxed_slice())
    }

    async fn decode(mut data: &[u8]) -> Result<Ipld, Self::Error> {
        Ipld::read_cbor(&mut data).await
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
    Cid(cid::Error),
    /// Ipld error.
    #[fail(display = "{}", _0)]
    Ipld(crate::error::IpldError),
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

impl From<cid::Error> for CborError {
    fn from(err: cid::Error) -> Self {
        Self::Cid(err)
    }
}

impl From<crate::error::IpldError> for CborError {
    fn from(err: crate::error::IpldError) -> Self {
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
    use crate::ipld;
    use crate::ipld::Cid;
    use async_std::task;

    async fn encode_decode_cbor() {
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": Cid::random(),
        });
        let bytes = DagCborCodec::encode(&ipld).await.unwrap();
        let ipld2 = DagCborCodec::decode(&bytes).await.unwrap();
        assert_eq!(ipld, ipld2);
    }

    #[test]
    fn test_encode_decode_cbor() {
        task::block_on(encode_decode_cbor());
    }
}
