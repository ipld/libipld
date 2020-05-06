//! `Ipld` codecs.
pub use crate::cid::Codec as Code;
use std::io::{Read, Write};

/// Codec trait.
pub trait Codec: Sized {
    /// Codec code.
    const CODE: Code;

    /// Error type.
    type Error: std::error::Error + Send + 'static;

    fn encode<T: Encode<Self> + ?Sized>(obj: &T) -> Result<Box<[u8]>, Self::Error> {
        let mut buf = Vec::new();
        obj.encode(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }

    fn decode<T: Decode<Self>>(mut bytes: &[u8]) -> Result<T, Self::Error> {
        T::decode(&mut bytes)
    }
}

pub trait Encode<C: Codec> {
    /// Encodes into a `impl Write`.
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), C::Error>;
}

pub trait Decode<C: Codec>: Sized {
    /// Decode from an `impl Read`.
    fn decode<R: Read>(r: &mut R) -> Result<Self, C::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld::Ipld;
    use thiserror::Error;

    struct CodecImpl;

    #[derive(Debug, Error)]
    enum CodecImplError {
        #[error("can only encode null")]
        NotNull,
        #[error("io: {0}")]
        Io(#[from] std::io::Error),
    }

    impl Codec for CodecImpl {
        const CODE: Code = Code::Raw;
        type Error = CodecImplError;
    }

    impl Encode<CodecImpl> for Ipld {
        fn encode<W: Write>(&self, w: &mut W) -> Result<(), <CodecImpl as Codec>::Error> {
            match self {
                Self::Null => Ok(w.write_all(&[0])?),
                _ => Err(CodecImplError::NotNull),
            }
        }
    }

    impl Decode<CodecImpl> for Ipld {
        fn decode<R: Read>(r: &mut R) -> Result<Self, <CodecImpl as Codec>::Error> {
            let mut buf = [0; 1];
            r.read_exact(&mut buf)?;
            if buf[0] == 0 {
                Ok(Ipld::Null)
            } else {
                Err(CodecImplError::NotNull)
            }
        }
    }

    #[test]
    fn test_codec() {
        let bytes = CodecImpl::encode(&Ipld::Null).unwrap();
        let ipld: Ipld = CodecImpl::decode(&bytes).unwrap();
        assert_eq!(ipld, Ipld::Null);
    }
}
