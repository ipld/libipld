//! CBOR codec.
use crate::codec::Codec;
use crate::error::Result;
use crate::ipld::{Ipld, IpldRef};

mod decode;
mod encode;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

impl Codec for DagCbor {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode<'a>(ipld: IpldRef<'a>) -> Result<Box<[u8]>> {
        let mut bytes = Vec::new();
        let mut enc = encode::Encoder::new(&mut bytes);
        enc.encode(ipld)?;
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
    use crate::{block, ipld};

    #[test]
    fn encode_decode_cbor() {
        let link = block!(null).unwrap();
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": link.cid(),
        });
        let ipld2 = DagCbor::decode(&DagCbor::encode(ipld.as_ref()).unwrap()).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
