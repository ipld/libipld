//! Jose codec.
//! TODO
#![deny(missing_docs)]
#![deny(warnings)]

mod codec;
mod error;

use std::collections::BTreeMap;

use libipld_cbor::DagCborCodec;
use libipld_core::cid::Cid;
use libipld_core::codec::Codec;
use libipld_core::codec::{Decode, Encode};
use libipld_core::error::UnsupportedCodec;
use libipld_core::ipld::Ipld;
use libipld_json::DagJsonCodec;

use codec::{Decoded, Encoded};

use crate::{
    codec::{DecodedRecipient, DecodedSignature, EncodedRecipient, EncodedSignature},
    error::Error,
};

/// DAG-JOSE codec
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DagJoseCodec;

impl Codec for DagJoseCodec {}

impl From<DagJoseCodec> for u64 {
    fn from(_: DagJoseCodec) -> Self {
        // Multicode comes from here https://github.com/multiformats/multicodec/blob/master/table.csv
        0x85
    }
}

impl TryFrom<u64> for DagJoseCodec {
    type Error = UnsupportedCodec;

    fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Encode<DagJoseCodec> for Ipld {
    fn encode<W: std::io::Write>(&self, _c: DagJoseCodec, w: &mut W) -> anyhow::Result<()> {
        self.encode(DagCborCodec, w)
    }
}
impl Decode<DagJoseCodec> for Ipld {
    fn decode<R: std::io::Read + std::io::Seek>(
        _c: DagJoseCodec,
        r: &mut R,
    ) -> anyhow::Result<Self> {
        Ipld::decode(DagCborCodec, r)
    }
}

/// A JSON Object Signging and Encryption value as defined in RFC7165.
#[derive(Clone, Debug, PartialEq)]
pub enum Jose {
    /// JSON Web Signature value
    Signature(JsonWebSignature),
    /// JSON Web Encryption value
    Encryption(JsonWebEncryption),
}

impl Encode<DagJoseCodec> for Jose {
    fn encode<W: std::io::Write>(&self, _c: DagJoseCodec, w: &mut W) -> anyhow::Result<()> {
        let encoded: Encoded = self.clone().try_into()?;
        encoded.encode(DagCborCodec, w)
    }
}
impl Decode<DagJoseCodec> for Jose {
    fn decode<R: std::io::Read + std::io::Seek>(
        _c: DagJoseCodec,
        r: &mut R,
    ) -> anyhow::Result<Self> {
        let encoded = Encoded::decode(DagCborCodec, r)?;
        match encoded.payload {
            Some(_) => Ok(Jose::Signature(encoded.try_into()?)),
            None => Ok(Jose::Encryption(encoded.try_into()?)),
        }
    }
}
// TODO put this behind feature flag
impl Encode<DagJsonCodec> for Jose {
    fn encode<W: std::io::Write>(&self, c: DagJsonCodec, w: &mut W) -> anyhow::Result<()> {
        match self {
            Jose::Signature(jws) => jws.encode(c, w),
            Jose::Encryption(jwe) => jwe.encode(c, w),
        }
    }
}

/// A JSON Web Signature object as defined in RFC7515.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonWebSignature {
    /// The payload base64 url encoded.
    // TODO Create a Base64Url encoded string type?
    pub payload: String,

    /// The set of signatures.
    pub signatures: Vec<Signature>,

    /// CID link from the payload.
    pub link: Cid,
}

impl Encode<DagJoseCodec> for JsonWebSignature {
    fn encode<W: std::io::Write>(&self, _c: DagJoseCodec, w: &mut W) -> anyhow::Result<()> {
        let encoded: Encoded = self.clone().try_into()?;
        encoded.encode(DagCborCodec, w)
    }
}
impl Decode<DagJoseCodec> for JsonWebSignature {
    fn decode<R: std::io::Read + std::io::Seek>(
        _c: DagJoseCodec,
        r: &mut R,
    ) -> anyhow::Result<Self> {
        Ok(Encoded::decode(DagCborCodec, r)?.try_into()?)
    }
}
// TODO put this behind feature flag
impl Encode<DagJsonCodec> for JsonWebSignature {
    fn encode<W: std::io::Write>(&self, c: DagJsonCodec, w: &mut W) -> anyhow::Result<()> {
        let decoded: Decoded = self.clone().try_into()?;
        // TODO: add direct conversion of Decoded type to Ipld
        let mut bytes = Vec::new();
        decoded.encode(DagCborCodec, &mut bytes)?;
        let data: Ipld = DagCborCodec.decode(&bytes)?;
        data.encode(c, w)
    }
}

impl TryFrom<Decoded> for JsonWebSignature {
    type Error = Error;

    fn try_from(mut value: Decoded) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: value.payload.ok_or(Error::NotJws)?,
            signatures: value.signatures.drain(..).map(Signature::from).collect(),
            link: value.link.ok_or(Error::NotJws)?,
        })
    }
}

impl TryFrom<Encoded> for JsonWebSignature {
    type Error = Error;

    fn try_from(value: Encoded) -> Result<Self, Self::Error> {
        let decoded: Decoded = value.into();
        decoded.try_into()
    }
}

/// A signature part of a JSON Web Signature.
#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    /// The optional unprotected header.
    pub header: BTreeMap<String, Ipld>,
    /// The protected header as a JSON object base64 url encoded.
    pub protected: Option<String>,
    /// The web signature base64 url encoded.
    pub signature: String,
}

impl From<DecodedSignature> for Signature {
    fn from(value: DecodedSignature) -> Self {
        Self {
            header: value.header,
            protected: value.protected,
            signature: value.signature,
        }
    }
}
impl From<EncodedSignature> for Signature {
    fn from(value: EncodedSignature) -> Self {
        let decoded: DecodedSignature = value.into();
        decoded.into()
    }
}

/// A JSON Web Encryption object as defined in RFC7516.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonWebEncryption {
    /// The optional additional authenticated data.
    pub aad: Option<String>,

    /// The ciphertext value resulting from authenticated encryption of the
    /// plaintext with additional authenticated data.
    pub ciphertext: String,

    /// Initialization Vector value used when encrypting the plaintext base64 url encoded.
    pub iv: String,

    /// The protected header as a JSON object base64 url encoded.
    pub protected: String,

    /// The set of recipients.
    pub recipients: Vec<Recipient>,

    /// The authentication tag value resulting from authenticated encryption.
    pub tag: String,

    /// The optional unprotected header.
    pub unprotected: BTreeMap<String, Ipld>,
}
impl Encode<DagJoseCodec> for JsonWebEncryption {
    fn encode<W: std::io::Write>(&self, _c: DagJoseCodec, w: &mut W) -> anyhow::Result<()> {
        let encoded: Encoded = self.clone().try_into()?;
        encoded.encode(DagCborCodec, w)
    }
}
impl Decode<DagJoseCodec> for JsonWebEncryption {
    fn decode<R: std::io::Read + std::io::Seek>(
        _c: DagJoseCodec,
        r: &mut R,
    ) -> anyhow::Result<Self> {
        Ok(Encoded::decode(DagCborCodec, r)?.try_into()?)
    }
}
// TODO put this behind feature flag
impl Encode<DagJsonCodec> for JsonWebEncryption {
    fn encode<W: std::io::Write>(&self, c: DagJsonCodec, w: &mut W) -> anyhow::Result<()> {
        let decoded: Decoded = self.clone().try_into()?;
        // TODO: add direct conversion of Decoded type to Ipld
        let mut bytes = Vec::new();
        decoded.encode(DagCborCodec, &mut bytes)?;
        let data: Ipld = DagCborCodec.decode(&bytes)?;
        data.encode(c, w)
    }
}

impl TryFrom<Decoded> for JsonWebEncryption {
    type Error = Error;

    fn try_from(mut value: Decoded) -> Result<Self, Self::Error> {
        Ok(Self {
            aad: value.aad,
            ciphertext: value.ciphertext.ok_or(Error::NotJwe)?,
            iv: value.iv.ok_or(Error::NotJwe)?,
            protected: value.protected.ok_or(Error::NotJwe)?,
            recipients: value.recipients.drain(..).map(Recipient::from).collect(),
            tag: value.tag.ok_or(Error::NotJwe)?,
            unprotected: value.unprotected,
        })
    }
}
impl TryFrom<Encoded> for JsonWebEncryption {
    type Error = Error;

    fn try_from(value: Encoded) -> Result<Self, Self::Error> {
        let decoded: Decoded = value.into();
        decoded.try_into()
    }
}

/// A recipient of a JSON Web Encryption message.
#[derive(Clone, Debug, PartialEq)]
pub struct Recipient {
    /// The encrypted content encryption key value.
    pub encrypted_key: Option<String>,

    /// The optional unprotected header.
    pub header: BTreeMap<String, Ipld>,
}

impl From<DecodedRecipient> for Recipient {
    fn from(value: DecodedRecipient) -> Self {
        Self {
            encrypted_key: value.encrypted_key,
            header: value.header,
        }
    }
}

impl From<EncodedRecipient> for Recipient {
    fn from(value: EncodedRecipient) -> Self {
        let decoded: DecodedRecipient = value.into();
        decoded.into()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    use libipld_core::{cid::Cid, codec::assert_roundtrip};
    use libipld_macro::ipld;

    fn fixture_jws() -> (Box<[u8]>, Box<[u8]>, Box<[u8]>) {
        let payload =
            base64_url::decode("AXESIIlVZVHDkmZ5zFLHLhgqVhkFakcnQJ7pOibQWtcnyhH0").unwrap();
        let protected = base64_url::decode("eyJhbGciOiJFZERTQSJ9").unwrap();
        let signature =  base64_url::decode("-_9J5OZcl5lVuRlgI1NJEzc0FqEb6_2yVskUaQPducRQ4oe-N5ynCl57wDm4SPtm1L1bltrphpQeBOeWjVW1BQ").unwrap();
        (
            payload.into_boxed_slice(),
            protected.into_boxed_slice(),
            signature.into_boxed_slice(),
        )
    }
    fn fixture_jws_base64(
        payload: &Box<[u8]>,
        protected: &Box<[u8]>,
        signature: &Box<[u8]>,
    ) -> (String, String, String) {
        (
            base64_url::encode(payload.as_ref()),
            base64_url::encode(protected.as_ref()),
            base64_url::encode(signature.as_ref()),
        )
    }
    fn fixture_jwe() -> (Box<[u8]>, Box<[u8]>, Box<[u8]>, Box<[u8]>) {
        let ciphertext = base64_url::decode("3XqLW28NHP-raqW8vMfIHOzko4N3IRaR").unwrap();
        let iv = base64_url::decode("PSWIuAyO8CpevzCL").unwrap();
        let protected = base64_url::decode("eyJhbGciOiJkaXIiLCJlbmMiOiJBMTI4R0NNIn0").unwrap();
        let tag = base64_url::decode("WZAMBblhzDCsQWOAKdlkSA").unwrap();
        (
            ciphertext.into_boxed_slice(),
            iv.into_boxed_slice(),
            protected.into_boxed_slice(),
            tag.into_boxed_slice(),
        )
    }
    fn fixture_jwe_base64(
        ciphertext: &Box<[u8]>,
        iv: &Box<[u8]>,
        protected: &Box<[u8]>,
        tag: &Box<[u8]>,
    ) -> (String, String, String, String) {
        (
            base64_url::encode(ciphertext.as_ref()),
            base64_url::encode(iv.as_ref()),
            base64_url::encode(protected.as_ref()),
            base64_url::encode(tag.as_ref()),
        )
    }
    #[test]
    fn roundtrip_jws() {
        let (payload, protected, signature) = fixture_jws();
        let (payload_b64, protected_b64, signature_b64) =
            fixture_jws_base64(&payload, &protected, &signature);
        let link = Cid::try_from(base64_url::decode(&payload_b64).unwrap()).unwrap();
        assert_roundtrip(
            DagJoseCodec,
            &JsonWebSignature {
                payload: payload_b64,
                signatures: vec![Signature {
                    header: BTreeMap::from([
                        ("k0".to_string(), Ipld::from("v0")),
                        ("k1".to_string(), Ipld::from(1)),
                    ]),
                    protected: Some(protected_b64),
                    signature: signature_b64,
                }],
                link,
            },
            &ipld!({
                "payload": payload,
                "signatures": [{
                    "header": {
                        "k0": "v0",
                        "k1": 1
                    },
                    "protected": protected,
                    "signature": signature,
                }],
            }),
        );
    }
    #[test]
    fn roundtrip_jwe() {
        let (ciphertext, iv, protected, tag) = fixture_jwe();
        let (ciphertext_b64, iv_b64, protected_b64, tag_b64) =
            fixture_jwe_base64(&ciphertext, &iv, &protected, &tag);
        assert_roundtrip(
            DagJoseCodec,
            &JsonWebEncryption {
                aad: None,
                ciphertext: ciphertext_b64,
                iv: iv_b64,
                protected: protected_b64,
                recipients: vec![],
                tag: tag_b64,
                unprotected: BTreeMap::new(),
            },
            &ipld!({
                "ciphertext": ciphertext,
                "iv": iv,
                "protected": protected,
                "tag": tag,
            }),
        );
    }
}
