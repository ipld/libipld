//! `Ipld` codecs.
use crate::cid::CidGeneric;
use crate::multihash;
use std::convert::TryFrom;
use std::io::{Read, Write};

/// A CID with the IPLD Codec code table and the Multihash code table.
pub type Cid = CidGeneric<IpldCodec, multihash::Code>;

/// The IPLD Codec code table.
///
/// This table only contains the IPLD Codecs that are implemented by the `libipld` library.
#[derive(Clone, Copy, Eq, Debug, Hash, Ord, PartialEq, PartialOrd)]
pub enum IpldCodec {
    /// Raw Codec.
    Raw = 0x55,
    /// DAG Protocol Buffer Codec.
    #[cfg(feature = "dag-pb")]
    DagPb = 0x70,
    /// DAG CBOR Codec.
    #[cfg(feature = "dag-cbor")]
    DagCbor = 0x71,
    /// DAG JSON Codec.
    #[cfg(feature = "dag-json")]
    DagJson = 0x0129,
}

impl From<IpldCodec> for u64 {
    /// Return the codec as integer value.
    fn from(codec: IpldCodec) -> Self {
        codec as _
    }
}

impl TryFrom<u64> for IpldCodec {
    type Error = String;

    /// Return the `IpldCodec` based on the integer value. Error if no matching code exists.
    fn try_from(raw: u64) -> Result<Self, Self::Error> {
        match raw {
            0x55 => Ok(IpldCodec::Raw),
            #[cfg(feature = "dag-pb")]
            0x70 => Ok(IpldCodec::DagPb),
            #[cfg(feature = "dag-cbor")]
            0x71 => Ok(IpldCodec::DagCbor),
            #[cfg(feature = "dag-json")]
            0x0129 => Ok(IpldCodec::DagJson),
            _ => Err(format!(r#"Cannot convert code "{:?}" to codec."#, raw)),
        }
    }
}

/// Codec trait.
pub trait Codec<C = IpldCodec>: Sized
where
    C: Copy + TryFrom<u64> + Into<u64>,
{
    /// Codec code.
    const CODE: C;

    /// Error type.
    type Error: std::error::Error + Send + 'static;

    /// Encodes an encodable type.
    fn encode<T: Encode<Self, C> + ?Sized>(obj: &T) -> Result<Box<[u8]>, Self::Error> {
        let mut buf = Vec::new();
        obj.encode(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }

    /// Decodes a decodable type.
    fn decode<T: Decode<Self, C>>(mut bytes: &[u8]) -> Result<T, Self::Error> {
        T::decode(&mut bytes)
    }
}

/// Encode trait.
pub trait Encode<O, C = IpldCodec>
where
    O: Codec<C>,
    C: Copy + TryFrom<u64> + Into<u64>,
{
    /// Encodes into a `impl Write`.
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), O::Error>;
}

/// Decode trait.
pub trait Decode<O, C = IpldCodec>: Sized
where
    O: Codec<C>,
    C: Copy + TryFrom<u64> + Into<u64>,
{
    /// Decode from an `impl Read`.
    fn decode<R: Read>(r: &mut R) -> Result<Self, O::Error>;
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

    impl Codec<IpldCodec> for CodecImpl {
        const CODE: IpldCodec = IpldCodec::Raw;
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
