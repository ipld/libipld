//! `Ipld` codecs.
use crate::error::{Result, UnsupportedCodec};
use crate::MAX_BLOCK_SIZE;
use core::convert::TryFrom;
use std::io::{Read, Write};

/// Codec trait.
pub trait Codec:
    Copy + Unpin + Send + Sync + 'static + Sized + TryFrom<u64, Error = UnsupportedCodec> + Into<u64>
{
    /// Encodes an encodable type.
    fn encode<T: Encode<Self> + ?Sized>(&self, obj: &T) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(MAX_BLOCK_SIZE);
        obj.encode(*self, &mut buf)?;
        Ok(buf)
    }

    /// Decodes a decodable type.
    fn decode<T: Decode<Self>>(&self, mut bytes: &[u8]) -> Result<T> {
        T::decode(*self, &mut bytes)
    }
}

/// Encode trait.
pub trait Encode<C: Codec> {
    /// Encodes into a `impl Write`.
    fn encode<W: Write>(&self, c: C, w: &mut W) -> Result<()>;
}

/// Decode trait.
pub trait Decode<C: Codec>: Sized {
    /// Decode from an `impl Read`.
    fn decode<R: Read>(c: C, r: &mut R) -> Result<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld::Ipld;
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("not null")]
    pub struct NotNull;

    #[derive(Clone, Copy, Debug)]
    struct CodecImpl;

    impl Codec for CodecImpl {}

    impl From<CodecImpl> for u64 {
        fn from(_: CodecImpl) -> Self {
            0
        }
    }

    impl TryFrom<u64> for CodecImpl {
        type Error = UnsupportedCodec;

        fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
            Ok(Self)
        }
    }

    impl Encode<CodecImpl> for Ipld {
        fn encode<W: Write>(&self, _: CodecImpl, w: &mut W) -> Result<()> {
            match self {
                Self::Null => Ok(w.write_all(&[0])?),
                _ => Err(NotNull.into()),
            }
        }
    }

    impl Decode<CodecImpl> for Ipld {
        fn decode<R: Read>(_: CodecImpl, r: &mut R) -> Result<Self> {
            let mut buf = [0; 1];
            r.read_exact(&mut buf)?;
            if buf[0] == 0 {
                Ok(Ipld::Null)
            } else {
                Err(NotNull.into())
            }
        }
    }

    #[test]
    fn test_codec() {
        let bytes = CodecImpl.encode(&Ipld::Null).unwrap();
        let ipld: Ipld = CodecImpl.decode(&bytes).unwrap();
        assert_eq!(ipld, Ipld::Null);
    }
}
