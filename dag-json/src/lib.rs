//! Json codec.
#![deny(missing_docs)]
#![deny(warnings)]

use libipld_core::codec::{Codec, Decode, Encode};
use libipld_core::ipld::Ipld;
use libipld_core::error::{Result, UnsupportedCodec};
use core::convert::TryFrom;
// TODO vmx 2020-05-28: Don't expose the `serde_json` error directly, but wrap it in a custom one
pub use serde_json::Error;
use std::io::{Read, Write};

mod codec;

/// Json codec.
#[derive(Clone, Copy, Debug)]
pub struct DagJsonCodec;

impl Codec for DagJsonCodec {
    fn decode_ipld(&self, mut bytes: &[u8]) -> Result<Ipld> {
        Ipld::decode(*self, &mut bytes)
    }
}

impl TryFrom<u64> for DagJsonCodec {
    type Error = UnsupportedCodec;

    fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Encode<DagJsonCodec> for Ipld {
    fn encode<W: Write>(&self, _: DagJsonCodec, w: &mut W) -> Result<()> {
        Ok(codec::encode(self, w)?)
    }
}

impl Decode<DagJsonCodec> for Ipld {
    fn decode<R: Read>(_: DagJsonCodec, r: &mut R) -> Result<Self> {
        Ok(codec::decode(r)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::{Cid, RAW};
    use libipld_core::multihash::{Multihash, MultihashDigest, SHA2_256};
    use std::collections::BTreeMap;

    #[test]
    fn encode_struct() {
        let digest = Multihash::new(SHA2_256, &b"block"[..]).unwrap();
        let cid = Cid::new_v1(RAW, digest.to_raw().unwrap());

        // Create a contact object that looks like:
        // Contact { name: "Hello World", details: CID }
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), Ipld::String("Hello World!".to_string()));
        map.insert("details".to_string(), Ipld::Link(cid.clone()));
        let contact = Ipld::Map(map);

        let contact_encoded = DagJsonCodec.encode(&contact).unwrap();
        println!("encoded: {:02x?}", contact_encoded);
        println!(
            "encoded string {}",
            std::str::from_utf8(&contact_encoded).unwrap()
        );

        assert_eq!(
            std::str::from_utf8(&contact_encoded).unwrap(),
            format!(
                r#"{{"details":{{"/":"{}"}},"name":"Hello World!"}}"#,
                base64::encode(cid.to_bytes()),
            )
        );

        let contact_decoded: Ipld = DagJsonCodec.decode(&contact_encoded).unwrap();
        assert_eq!(contact_decoded, contact);
    }
}
