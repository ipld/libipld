//! IPLD Codecs.
#[cfg(feature = "dag-cbor")]
use crate::cbor::{DagCborCodec, Error as CborError};
use crate::codec::{Codec, Decode, Encode};
use crate::error::Error;
#[cfg(feature = "dag-json")]
use crate::json::{DagJsonCodec, Error as JsonError};
#[cfg(feature = "dag-pb")]
use crate::pb::{DagPbCodec, Error as PbError};
use crate::raw::{RawCodec, RawError};
use core::convert::TryFrom;
use std::io::{Read, Write};
use thiserror::Error;

/// Default codecs.
#[derive(Clone, Copy, Debug)]
pub enum IpldCodec {
    /// Raw codec.
    Raw,
    /// Cbor codec.
    #[cfg(feature = "dag-cbor")]
    Cbor,
    /// Json codec.
    #[cfg(feature = "dag-json")]
    Json,
    /// Protobuf codec.
    #[cfg(feature = "dag-pb")]
    Pb,
}

impl TryFrom<u64> for IpldCodec {
    type Error = Error;

    fn try_from(ccode: u64) -> Result<Self, Self::Error> {
        Ok(match ccode {
            crate::cid::RAW => Self::Raw,
            #[cfg(feature = "dag-cbor")]
            crate::cid::DAG_CBOR => Self::Cbor,
            #[cfg(feature = "dag-json")]
            crate::cid::DAG_JSON => Self::Json,
            #[cfg(feature = "dag-pb")]
            crate::cid::DAG_PB => Self::Pb,
            _ => return Err(Error::UnsupportedCodec(ccode)),
        })
    }
}

impl Codec for IpldCodec {
    type Error = IpldCodecError;
}

/// Wrapper
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wrapper<T>(pub T);

//#[cfg(all(feature = "dag-cbor", feature = "dag-json", feature = "dag-pb"))]
impl<T> Encode<IpldCodec> for Wrapper<T>
where
    T: Encode<RawCodec>,
//    #[cfg(feature = "dag-cbor")]
//    T: Encode<DagCborCodec>,
//    #[cfg(feature = "dag-json")]
//    T: Encode<DagJsonCodec>,
//    #[cfg(feature = "dag-pb")]
//    T: Encode<DagPbCodec>,
{
    fn encode<W: Write>(&self, c: IpldCodec, w: &mut W) -> Result<(), <IpldCodec as Codec>::Error> {
        match c {
            IpldCodec::Raw => self.0.encode(RawCodec, w)?,
            #[cfg(feature = "dag-cbor")]
            IpldCodec::Cbor => self.0.encode(DagCborCodec, w)?,
            #[cfg(feature = "dag-json")]
            IpldCodec::Json => self.0.encode(DagJsonCodec, w)?,
            #[cfg(feature = "dag-pb")]
            IpldCodec::Pb => self.0.encode(DagPbCodec, w)?,
        };
        Ok(())
    }
}

impl<T> Decode<IpldCodec> for Wrapper<T>
where
    T: Decode<RawCodec>,
//    #[cfg(feature = "dag-cbor")]
//    T: Decode<DagCborCodec>,
//    #[cfg(feature = "dag-json")]
//    T: Decode<DagJsonCodec>,
//    #[cfg(feature = "dag-pb")]
//    T: Decode<DagPbCodec>,
{
    fn decode<R: Read>(c: IpldCodec, r: &mut R) -> Result<Self, <IpldCodec as Codec>::Error> {
        Ok(Wrapper(match c {
            IpldCodec::Raw => T::decode(RawCodec, r)?,
            #[cfg(feature = "dag-cbor")]
            IpldCodec::Cbor => T::decode(DagCborCodec, r)?,
            #[cfg(feature = "dag-json")]
            IpldCodec::Json => T::decode(DagJsonCodec, r)?,
            #[cfg(feature = "dag-pb")]
            IpldCodec::Pb => T::decode(DagPbCodec, r)?,
        }))
    }
}

/// Errors that happen within the [`EncodeDecodeIpld`] implementation of [`IpldCodec`].
#[derive(Debug, Error)]
pub enum IpldCodecError {
    /// [Raw Codec](raw::RawCodec) error.
    #[error("Raw Codec: {0}")]
    Raw(#[from] RawError),

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

/*
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
}*/
