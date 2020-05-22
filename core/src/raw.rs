//! Implements the raw codec.
use crate::codec::{Code, Codec, Decode, Encode};
use crate::error::{TypeError, TypeErrorType};
use crate::ipld::Ipld;
use std::io::{Read, Write};
use thiserror::Error;

/// Raw codec.
pub struct RawCodec;

impl Codec for RawCodec {
    const CODE: Code = Code::Raw;

    type Error = RawError;
}

/// Error returned by the `RawCodec`.
#[derive(Debug, Error)]
pub enum RawError {
    /// Io error.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// Type error.
    #[error("{0}")]
    Type(#[from] TypeError),
}

impl Encode<RawCodec> for [u8] {
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), RawError> {
        Ok(w.write_all(self)?)
    }
}

impl Encode<RawCodec> for Box<[u8]> {
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), RawError> {
        Ok(w.write_all(&self[..])?)
    }
}

impl Encode<RawCodec> for Vec<u8> {
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), RawError> {
        Ok(w.write_all(&self[..])?)
    }
}

impl Encode<RawCodec> for Ipld {
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), RawError> {
        if let Ipld::Bytes(bytes) = self {
            bytes.encode(w)
        } else {
            Err(TypeError::new(TypeErrorType::Bytes, self).into())
        }
    }
}

impl Decode<RawCodec> for Box<[u8]> {
    fn decode<R: Read>(r: &mut R) -> Result<Self, RawError> {
        let buf: Vec<u8> = Decode::<RawCodec>::decode(r)?;
        Ok(buf.into_boxed_slice())
    }
}

impl Decode<RawCodec> for Vec<u8> {
    fn decode<R: Read>(r: &mut R) -> Result<Self, RawError> {
        let mut buf = vec![];
        r.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

impl Decode<RawCodec> for Ipld {
    fn decode<R: Read>(r: &mut R) -> Result<Self, RawError> {
        let bytes: Vec<u8> = Decode::<RawCodec>::decode(r)?;
        Ok(Ipld::Bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_codec() {
        let data: &[u8] = &[0, 1, 2, 3];
        let bytes = RawCodec::encode(data).unwrap();
        assert_eq!(data, &*bytes);
        let data2: Vec<u8> = RawCodec::decode(&bytes).unwrap();
        assert_eq!(data, &*data2);

        let ipld = Ipld::Bytes(data2);
        let bytes = RawCodec::encode(&ipld).unwrap();
        assert_eq!(data, &*bytes);
        let ipld2: Ipld = RawCodec::decode(&bytes).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
