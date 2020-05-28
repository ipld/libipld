//! Json codec.
#![deny(missing_docs)]
#![deny(warnings)]

use libipld_core::codec::{Code, Codec, Decode, Encode};
use libipld_core::ipld::Ipld;
// TODO vmx 2020-05-28: Don't expose the `serde_json` error directly, but wrap it in a custom one
pub use serde_json::Error;
use std::convert::TryFrom;
use std::io::{Read, Write};

mod codec;

/// Json codec.
#[derive(Clone, Copy, Debug)]
pub struct DagJsonCodec;

impl Codec for DagJsonCodec {
    const CODE: Code = Code::DagJSON;

    type Error = Error;
}

impl<C, H> Encode<DagJsonCodec> for Ipld<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        codec::encode(self, w)
    }
}

impl<C, H> Decode<DagJsonCodec> for Ipld<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn decode<R: Read>(r: &mut R) -> Result<Self, Error> {
        codec::decode(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::multihash::Sha2_256;
    use std::collections::BTreeMap;

    #[test]
    fn encode_struct() {
        let digest = Sha2_256::digest(b"block");
        let cid = Cid::new_v0(digest).unwrap();

        // Create a contact object that looks like:
        // Contact { name: "Hello World", details: CID }
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), Ipld::String("Hello World!".to_string()));
        map.insert("details".to_string(), Ipld::Link(cid.clone()));
        let contact = Ipld::Map(map);

        let contact_encoded = DagJsonCodec::encode(&contact).unwrap();
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

        let contact_decoded: Ipld = DagJsonCodec::decode(&contact_encoded).unwrap();
        assert_eq!(contact_decoded, contact);
    }
}
