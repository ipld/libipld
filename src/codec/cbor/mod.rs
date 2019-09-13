//! CBOR codec.
use crate::codec::Codec;
use crate::error::Result;
use crate::ipld::Ipld;

pub mod decode;
pub mod encode;

pub use encode::WriteCbor;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCborCodec;

impl Codec for DagCborCodec {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode(ipld: &Ipld) -> Result<Box<[u8]>> {
        let mut bytes = Vec::new();
        ipld.write_cbor(&mut bytes)?;
        Ok(bytes.into_boxed_slice())
    }

    fn decode(data: &[u8]) -> Result<Ipld> {
        let mut dec = decode::Decoder::new(data);
        let ipld = dec.decode()?;
        Ok(ipld)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld;
    use cid::Cid;

    #[test]
    fn encode_decode_cbor() {
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": Cid::random(),
        });
        let ipld2 = DagCborCodec::decode(&DagCborCodec::encode(&ipld).unwrap()).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
