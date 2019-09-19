//! CBOR codec.
use crate::codec::Codec;
use crate::error::Result;
use crate::ipld::Ipld;
use async_trait::async_trait;

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

    async fn encode(ipld: &Ipld) -> Result<Box<[u8]>> {
        let mut bytes = Vec::new();
        ipld.write_cbor(&mut bytes).await?;
        Ok(bytes.into_boxed_slice())
    }

    async fn decode(mut data: &[u8]) -> Result<Ipld> {
        Ipld::read_cbor(&mut data).await
    }
}

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
