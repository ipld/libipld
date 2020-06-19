//! IPLD Codecs.
#[cfg(feature = "dag-cbor")]
use crate::cbor::{DagCborCodec, Error as CborError};
use crate::ipld::Ipld;
#[cfg(feature = "dag-json")]
use crate::json::{DagJsonCodec, Error as JsonError};
use crate::multihash::{Code as HCode, MultihashCode};
#[cfg(feature = "dag-pb")]
use crate::pb::{DagPbCodec, Error as PbError};
use crate::raw;
use crate::IpldCodec;
use libipld_core::codec::Codec;
use std::convert::TryFrom;
use thiserror::Error;

/// The `EncodeDecodeIpld` trait allows to encode/decode [`Ipld`] objects.
///
/// It is usually implemented by IPLD Codec code tables. This way [`Codec`] implementations can
/// be mapped to specific codes.
///
/// # Example
///
/// ```
/// use std::convert::TryFrom;
/// use std::error::Error;
/// use std::fmt;
///
/// use libipld::encode_decode::EncodeDecodeIpld;
/// use libipld_core::codec::Codec;
/// use libipld_core::ipld::Ipld;
/// use libipld_core::raw;
///
/// #[derive(Clone, Copy, Debug)]
/// pub enum IpldCodec {
///     Raw = 0x55,
/// }
///
/// impl EncodeDecodeIpld for IpldCodec {
///     /// Error type.
///     type Error = EncodeDecodeError;
///
///     /// Encodes an encodable type.
///     fn encode(&self, obj: &Ipld<IpldCodec>) -> Result<Box<[u8]>, EncodeDecodeError> {
///         match self {
///             Self::Raw => raw::RawCodec::encode(obj)
///                 .map_err(|err| EncodeDecodeError(format!("{:?}", err))),
///         }
///     }
///
///     /// Decodes a decodable type.
///     fn decode(&self, bytes: &[u8]) -> Result<Ipld<IpldCodec>, EncodeDecodeError> {
///         match self {
///             Self::Raw => raw::RawCodec::decode(&bytes)
///                 .map_err(|err| EncodeDecodeError(format!("{:?}", err))),
///         }
///     }
/// }
///
/// let encoded = IpldCodec::Raw
///     .encode(&Ipld::<IpldCodec>::Bytes(vec![0x11, 0x22]))
///     .unwrap();
/// let expected: Box<[u8]> = Box::new([0x11, 0x22]);
/// assert_eq!(encoded, expected);
///
/// // The rest of the code is only needed to have a working example
///
/// impl From<IpldCodec> for u64 {
///     /// Return the codec as integer value.
///     fn from(codec: IpldCodec) -> Self {
///         codec as _
///     }
/// }
///
/// impl TryFrom<u64> for IpldCodec {
///     type Error = String;
///
///     /// Return the `IpldCodec` based on the integer value. Error if no matching code exists.
///     fn try_from(raw: u64) -> Result<Self, Self::Error> {
///         match raw {
///             0x55 => Ok(IpldCodec::Raw),
///             _ => Err("Cannot convert code to codec.".to_string()),
///         }
///     }
/// }
///
/// #[derive(Debug)]
/// pub struct EncodeDecodeError(String);
/// impl fmt::Display for EncodeDecodeError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "error: {}", &self.0)
///     }
/// }
/// impl Error for EncodeDecodeError {}
/// ```
pub trait EncodeDecodeIpld<H = HCode>
where
    Self: Copy + TryFrom<u64> + Into<u64>,
    H: MultihashCode,
{
    /// Error type.
    type Error: std::error::Error + Send + 'static;

    /// Encodes an `Ipld` object as bytes.
    fn encode(
        &self,
        obj: &Ipld<Self, H>,
    ) -> Result<Box<[u8]>, <Self as EncodeDecodeIpld<H>>::Error>;

    /// Decodes bytes into an `Ipld` object.
    fn decode(&self, bytes: &[u8]) -> Result<Ipld<Self, H>, <Self as EncodeDecodeIpld<H>>::Error>;
}

/// Errors that happen within the [`EncodeDecodeIpld`] implementation of [`IpldCodec`].
#[derive(Debug, Error)]
pub enum EncodeDecodeError {
    /// [Raw Codec](raw::RawCodec) error.
    #[error("Raw Codec: {0}")]
    Raw(#[from] raw::RawError),

    #[cfg(feature = "dag-cbor")]
    /// [DAG-CBOR Codec](DagCborCodec) error.
    #[error("DAG-CBOR Codec: {0}")]
    Cbor(#[from] CborError),

    /// [DAG-JSON Codec](DagJsonCodec) error.
    #[cfg(feature = "dag-json")]
    #[error("DAG-JSON Codec: {0}")]
    Json(#[from] JsonError),

    /// [DAG-PB Codec](DagPbCodec) error.
    #[cfg(feature = "dag-pb")]
    #[error("DAG-PB Codec: {0}")]
    Pb(#[from] PbError),
}

impl EncodeDecodeIpld for IpldCodec {
    /// Error type.
    type Error = EncodeDecodeError;

    /// Encodes an encodable type.
    fn encode(&self, obj: &Ipld<IpldCodec>) -> Result<Box<[u8]>, EncodeDecodeError> {
        match self {
            Self::Raw => raw::RawCodec::encode(obj).map_err(|err| err.into()),
            #[cfg(feature = "dag-cbor")]
            Self::DagCbor => DagCborCodec::encode(obj).map_err(|err| err.into()),
            #[cfg(feature = "dag-json")]
            Self::DagJson => DagJsonCodec::encode(obj).map_err(|err| err.into()),
            #[cfg(feature = "dag-pb")]
            Self::DagPb => DagPbCodec::encode(obj).map_err(|err| err.into()),
        }
    }

    /// Decodes a decodable type.
    fn decode(&self, bytes: &[u8]) -> Result<Ipld<IpldCodec>, EncodeDecodeError> {
        match self {
            Self::Raw => raw::RawCodec::decode(&bytes).map_err(|err| err.into()),
            #[cfg(feature = "dag-cbor")]
            Self::DagCbor => DagCborCodec::decode(bytes).map_err(|err| err.into()),
            #[cfg(feature = "dag-json")]
            Self::DagJson => DagJsonCodec::decode(bytes).map_err(|err| err.into()),
            #[cfg(feature = "dag-pb")]
            Self::DagPb => DagPbCodec::decode(bytes).map_err(|err| err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_encode() {
        let data = Ipld::Bytes([0x22, 0x33, 0x44].to_vec());
        let result = IpldCodec::Raw.encode(&data).unwrap();
        assert_eq!(result, vec![0x22, 0x33, 0x44].into_boxed_slice());
    }

    #[test]
    fn raw_decode() {
        let data = [0x22, 0x33, 0x44];
        let result = IpldCodec::Raw.decode(&data).unwrap();
        assert_eq!(result, Ipld::Bytes(data.to_vec()));
    }

    #[cfg(feature = "dag-cbor")]
    #[test]
    fn dag_cbor_encode() {
        let data = Ipld::Bytes([0x22, 0x33, 0x44].to_vec());
        let result = IpldCodec::DagCbor.encode(&data).unwrap();
        assert_eq!(result, vec![0x43, 0x22, 0x33, 0x44].into_boxed_slice());
    }

    #[cfg(feature = "dag-cbor")]
    #[test]
    fn dag_cbor_decode() {
        let data = [0x43, 0x22, 0x33, 0x44];
        let result = IpldCodec::DagCbor.decode(&data).unwrap();
        assert_eq!(result, Ipld::Bytes(vec![0x22, 0x33, 0x44]));
    }

    #[cfg(feature = "dag-json")]
    #[test]
    fn dag_json_encode() {
        let data = Ipld::Bool(true);
        let result = String::from_utf8(IpldCodec::DagJson.encode(&data).unwrap().to_vec()).unwrap();
        assert_eq!(result, "true");
    }

    #[cfg(feature = "dag-json")]
    #[test]
    fn dag_json_decode() {
        let data = b"true";
        let result = IpldCodec::DagJson.decode(data).unwrap();
        assert_eq!(result, Ipld::Bool(true));
    }

    #[cfg(feature = "dag-pb")]
    #[test]
    fn dag_pb_encode() {
        let mut data_map = std::collections::BTreeMap::<String, Ipld>::new();
        data_map.insert("Data".to_string(), Ipld::Bytes(b"data".to_vec()));
        data_map.insert("Links".to_string(), Ipld::List(vec![]));

        let data = Ipld::Map(data_map);
        let result = IpldCodec::DagPb.encode(&data).unwrap();
        assert_eq!(
            result,
            vec![0x0a, 0x04, 0x64, 0x61, 0x74, 0x61].into_boxed_slice()
        );
    }

    #[cfg(feature = "dag-pb")]
    #[test]
    fn dag_pb_decode() {
        let mut data_map = std::collections::BTreeMap::<String, Ipld>::new();
        data_map.insert("Data".to_string(), Ipld::Bytes(b"data".to_vec()));
        data_map.insert("Links".to_string(), Ipld::List(vec![]));
        let expected = Ipld::Map(data_map);

        let data = [0x0a, 0x04, 0x64, 0x61, 0x74, 0x61];
        let result = IpldCodec::DagPb.decode(&data).unwrap();
        assert_eq!(result, expected);
    }
}
