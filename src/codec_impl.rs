//! IPLD Codecs.
#[cfg(feature = "dag-cbor")]
use crate::cbor::DagCborCodec;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{Result, UnsupportedCodec};
use crate::ipld::Ipld;
#[cfg(feature = "dag-json")]
use crate::json::DagJsonCodec;
#[cfg(feature = "dag-pb")]
use crate::pb::DagPbCodec;
use crate::raw::RawCodec;
use core::convert::TryFrom;
use std::io::{Read, Write};

/// Default codecs.
#[derive(Clone, Copy, Debug)]
pub enum Multicodec {
    /// Raw codec.
    Raw,
    /// Cbor codec.
    #[cfg(feature = "dag-cbor")]
    DagCbor,
    /// Json codec.
    #[cfg(feature = "dag-json")]
    DagJson,
    /// Protobuf codec.
    #[cfg(feature = "dag-pb")]
    DagPb,
}

impl TryFrom<u64> for Multicodec {
    type Error = UnsupportedCodec;

    fn try_from(ccode: u64) -> core::result::Result<Self, Self::Error> {
        Ok(match ccode {
            crate::cid::RAW => Self::Raw,
            #[cfg(feature = "dag-cbor")]
            crate::cid::DAG_CBOR => Self::DagCbor,
            #[cfg(feature = "dag-json")]
            crate::cid::DAG_JSON => Self::DagJson,
            #[cfg(feature = "dag-pb")]
            crate::cid::DAG_PROTOBUF => Self::DagPb,
            _ => return Err(UnsupportedCodec(ccode)),
        })
    }
}

impl From<Multicodec> for u64 {
    fn from(mc: Multicodec) -> Self {
        match mc {
            Multicodec::Raw => crate::cid::RAW,
            #[cfg(feature = "dag-cbor")]
            Multicodec::DagCbor => crate::cid::DAG_CBOR,
            #[cfg(feature = "dag-json")]
            Multicodec::DagJson => crate::cid::DAG_JSON,
            #[cfg(feature = "dag-pb")]
            Multicodec::DagPb => crate::cid::DAG_PROTOBUF,
        }
    }
}

impl From<RawCodec> for Multicodec {
    fn from(_: RawCodec) -> Self {
        Self::Raw
    }
}

#[cfg(feature = "dag-cbor")]
impl From<DagCborCodec> for Multicodec {
    fn from(_: DagCborCodec) -> Self {
        Self::DagCbor
    }
}

#[cfg(feature = "dag-cbor")]
impl From<Multicodec> for DagCborCodec {
    fn from(_: Multicodec) -> Self {
        Self
    }
}

#[cfg(feature = "dag-json")]
impl From<DagJsonCodec> for Multicodec {
    fn from(_: DagJsonCodec) -> Self {
        Self::DagJson
    }
}

#[cfg(feature = "dag-json")]
impl From<Multicodec> for DagJsonCodec {
    fn from(_: Multicodec) -> Self {
        Self
    }
}

#[cfg(feature = "dag-pb")]
impl From<DagPbCodec> for Multicodec {
    fn from(_: DagPbCodec) -> Self {
        Self::DagPb
    }
}

#[cfg(feature = "dag-pb")]
impl From<Multicodec> for DagPbCodec {
    fn from(_: Multicodec) -> Self {
        Self
    }
}

impl Codec for Multicodec {
    fn decode_ipld(&self, mut bytes: &[u8]) -> Result<Ipld> {
        Ipld::decode(*self, &mut bytes)
    }
}

impl Encode<Multicodec> for Ipld {
    fn encode<W: Write>(&self, c: Multicodec, w: &mut W) -> Result<()> {
        match c {
            Multicodec::Raw => self.encode(RawCodec, w)?,
            #[cfg(feature = "dag-cbor")]
            Multicodec::DagCbor => self.encode(DagCborCodec, w)?,
            #[cfg(feature = "dag-json")]
            Multicodec::DagJson => self.encode(DagJsonCodec, w)?,
            #[cfg(feature = "dag-pb")]
            Multicodec::DagPb => self.encode(DagPbCodec, w)?,
        };
        Ok(())
    }
}

impl Decode<Multicodec> for Ipld {
    fn decode<R: Read>(c: Multicodec, r: &mut R) -> Result<Self> {
        Ok(match c {
            Multicodec::Raw => Self::decode(RawCodec, r)?,
            #[cfg(feature = "dag-cbor")]
            Multicodec::DagCbor => Self::decode(DagCborCodec, r)?,
            #[cfg(feature = "dag-json")]
            Multicodec::DagJson => Self::decode(DagJsonCodec, r)?,
            #[cfg(feature = "dag-pb")]
            Multicodec::DagPb => Self::decode(DagPbCodec, r)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_encode() {
        let data = Ipld::Bytes([0x22, 0x33, 0x44].to_vec());
        let result = Multicodec::Raw.encode(&data).unwrap();
        assert_eq!(result, vec![0x22, 0x33, 0x44].into_boxed_slice());
    }

    #[test]
    fn raw_decode() {
        let data = [0x22, 0x33, 0x44];
        let result: Ipld = Multicodec::Raw.decode(&data).unwrap();
        assert_eq!(result, Ipld::Bytes(data.to_vec()));
    }

    #[cfg(feature = "dag-cbor")]
    #[test]
    fn dag_cbor_encode() {
        let data = Ipld::Bytes([0x22, 0x33, 0x44].to_vec());
        let result = Multicodec::DagCbor.encode(&data).unwrap();
        assert_eq!(result, vec![0x43, 0x22, 0x33, 0x44].into_boxed_slice());
    }

    #[cfg(feature = "dag-cbor")]
    #[test]
    fn dag_cbor_decode() {
        let data = [0x43, 0x22, 0x33, 0x44];
        let result: Ipld = Multicodec::DagCbor.decode(&data).unwrap();
        assert_eq!(result, Ipld::Bytes(vec![0x22, 0x33, 0x44]));
    }

    #[cfg(feature = "dag-json")]
    #[test]
    fn dag_json_encode() {
        let data = Ipld::Bool(true);
        let result =
            String::from_utf8(Multicodec::DagJson.encode(&data).unwrap().to_vec()).unwrap();
        assert_eq!(result, "true");
    }

    #[cfg(feature = "dag-json")]
    #[test]
    fn dag_json_decode() {
        let data = b"true";
        let result: Ipld = Multicodec::DagJson.decode(data).unwrap();
        assert_eq!(result, Ipld::Bool(true));
    }

    #[cfg(feature = "dag-pb")]
    #[test]
    fn dag_pb_encode() {
        let mut data_map = std::collections::BTreeMap::<String, Ipld>::new();
        data_map.insert("Data".to_string(), Ipld::Bytes(b"data".to_vec()));
        data_map.insert("Links".to_string(), Ipld::List(vec![]));

        let data = Ipld::Map(data_map);
        let result = Multicodec::DagPb.encode(&data).unwrap();
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
        let result: Ipld = Multicodec::DagPb.decode(&data).unwrap();
        assert_eq!(result, expected);
    }
}
