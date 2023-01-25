//! Codec provides two flavors of structures for encoding and decoding.
//!
//! Encoded* structures represent structures that use binary data
//! Decoded* structures represent structures that use base64 data
//!
//! From implementation are provided between Encoded and Decoded types.
#![deny(missing_docs)]
#![deny(warnings)]

use std::collections::BTreeMap;

use libipld_cbor::Bytes;
use libipld_cbor_derive::DagCborInternal;
use libipld_core::cid::Cid;
use libipld_core::ipld::Ipld;

use crate::{error::Error, JsonWebEncryption};
use crate::{Jose, Signature};
use crate::{JsonWebSignature, Recipient};

/// Encoded represents the union of fields from JSON Web Signature (JWS)
/// and JSON Web Encryption (JWE) objects.
///
/// The data is repsented as bytes and therefore can be encoded
/// into a DAG-JOSE object using DAG-CBOR.
///
/// See https://ipld.io/specs/codecs/dag-jose/spec/#format
#[derive(PartialEq, Default, Debug, DagCborInternal)]
pub struct Encoded {
    // JWS fields
    #[ipld(default = None)]
    pub payload: Option<Bytes>,
    #[ipld(default = Vec::new())]
    pub signatures: Vec<EncodedSignature>,

    // JWE fields
    #[ipld(default = None)]
    pub aad: Option<Bytes>,
    #[ipld(default = None)]
    pub ciphertext: Option<Bytes>,
    #[ipld(default = None)]
    pub iv: Option<Bytes>,
    #[ipld(default = None)]
    pub protected: Option<Bytes>,
    #[ipld(default = Vec::new())]
    pub recipients: Vec<EncodedRecipient>,
    #[ipld(default = None)]
    pub tag: Option<Bytes>,
    #[ipld(default = BTreeMap::new())]
    pub unprotected: BTreeMap<String, Ipld>,
}

impl TryFrom<Decoded> for Encoded {
    type Error = Error;

    fn try_from(mut value: Decoded) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: Option::from_base64(value.payload)?,
            signatures: value
                .signatures
                .drain(..)
                .map(EncodedSignature::try_from)
                .collect::<Result<Vec<EncodedSignature>, Self::Error>>()?,
            aad: Option::from_base64(value.aad)?,
            ciphertext: Option::from_base64(value.ciphertext)?,
            iv: Option::from_base64(value.iv)?,
            protected: Option::from_base64(value.protected)?,
            recipients: value
                .recipients
                .drain(..)
                .map(EncodedRecipient::try_from)
                .collect::<Result<Vec<EncodedRecipient>, Self::Error>>()?,
            tag: Option::from_base64(value.tag)?,
            unprotected: value.unprotected,
        })
    }
}

impl TryFrom<JsonWebSignature> for Encoded {
    type Error = Error;

    fn try_from(value: JsonWebSignature) -> Result<Self, Self::Error> {
        let decoded: Decoded = value.into();
        decoded.try_into()
    }
}

impl TryFrom<JsonWebEncryption> for Encoded {
    type Error = Error;

    fn try_from(value: JsonWebEncryption) -> Result<Self, Self::Error> {
        let decoded: Decoded = value.into();
        decoded.try_into()
    }
}
impl TryFrom<Jose> for Encoded {
    type Error = Error;

    fn try_from(value: Jose) -> Result<Self, Self::Error> {
        let decoded: Decoded = value.into();
        decoded.try_into()
    }
}

#[derive(PartialEq, Default, Debug, DagCborInternal)]
pub struct EncodedSignature {
    #[ipld(default = BTreeMap::new())]
    header: BTreeMap<String, Ipld>,
    #[ipld(default = None)]
    protected: Option<Bytes>,
    signature: Bytes,
}

impl TryFrom<DecodedSignature> for EncodedSignature {
    type Error = Error;

    fn try_from(value: DecodedSignature) -> Result<Self, Self::Error> {
        Ok(Self {
            header: value.header,
            protected: Option::from_base64(value.protected)?,
            signature: Bytes::from_base64(value.signature)?,
        })
    }
}

#[derive(PartialEq, Default, Debug, DagCborInternal)]
pub struct EncodedRecipient {
    #[ipld(default = None)]
    encrypted_key: Option<Bytes>,
    #[ipld(default = BTreeMap::new())]
    header: BTreeMap<String, Ipld>,
}

impl TryFrom<DecodedRecipient> for EncodedRecipient {
    type Error = Error;

    fn try_from(value: DecodedRecipient) -> Result<Self, Self::Error> {
        Ok(Self {
            encrypted_key: Option::from_base64(value.encrypted_key)?,
            header: value.header,
        })
    }
}

/// Decoded represents the union of fields from JSON Web Signature (JWS)
/// and JSON Web Encryption (JWE) objects.
///
/// The data is repsented as base64 URL encoded strings and enabling
/// direct conversion into a publicly exposed struct.
///
/// See https://ipld.io/specs/codecs/dag-jose/spec/#decoded-jose
#[derive(PartialEq, Default, Debug, DagCborInternal)]
pub struct Decoded {
    // JWS fields
    #[ipld(default = None)]
    pub payload: Option<String>,
    #[ipld(default = Vec::new())]
    pub signatures: Vec<DecodedSignature>,
    #[ipld(default = None)]
    pub link: Option<Cid>,

    // JWE fields
    #[ipld(default = None)]
    pub aad: Option<String>,
    #[ipld(default = None)]
    pub ciphertext: Option<String>,
    #[ipld(default = None)]
    pub iv: Option<String>,
    #[ipld(default = None)]
    pub protected: Option<String>,
    #[ipld(default = Vec::new())]
    pub recipients: Vec<DecodedRecipient>,
    #[ipld(default = None)]
    pub tag: Option<String>,
    #[ipld(default = BTreeMap::new())]
    pub unprotected: BTreeMap<String, Ipld>,
}

impl From<Encoded> for Decoded {
    fn from(mut value: Encoded) -> Self {
        let link = value
            .payload
            .as_ref()
            .map(|v| Cid::try_from(&**v))
            .transpose()
            .expect("TODO");
        Self {
            payload: value.payload.to_base64(),
            signatures: value
                .signatures
                .drain(..)
                .map(DecodedSignature::from)
                .collect(),
            link,
            aad: value.aad.to_base64(),
            ciphertext: value.ciphertext.to_base64(),
            iv: value.iv.to_base64(),
            protected: value.protected.to_base64(),
            recipients: value
                .recipients
                .drain(..)
                .map(DecodedRecipient::from)
                .collect(),
            tag: value.tag.to_base64(),
            unprotected: value.unprotected,
        }
    }
}

impl From<JsonWebSignature> for Decoded {
    fn from(mut value: JsonWebSignature) -> Self {
        Self {
            payload: Some(value.payload),
            signatures: value
                .signatures
                .drain(..)
                .map(DecodedSignature::from)
                .collect(),
            link: Some(value.link),
            aad: None,
            ciphertext: None,
            iv: None,
            protected: None,
            recipients: vec![],
            tag: None,
            unprotected: BTreeMap::new(),
        }
    }
}
impl From<JsonWebEncryption> for Decoded {
    fn from(mut value: JsonWebEncryption) -> Self {
        Self {
            payload: None,
            signatures: vec![],
            link: None,
            aad: value.aad,
            ciphertext: Some(value.ciphertext),
            iv: Some(value.iv),
            protected: Some(value.protected),
            recipients: value
                .recipients
                .drain(..)
                .map(DecodedRecipient::from)
                .collect(),
            tag: Some(value.tag),
            unprotected: value.unprotected,
        }
    }
}
impl From<Jose> for Decoded {
    fn from(value: Jose) -> Self {
        match value {
            Jose::Signature(jws) => Decoded::from(jws),
            Jose::Encryption(jwe) => Decoded::from(jwe),
        }
    }
}

/// Decoded form of a JWS signature
#[derive(PartialEq, Default, Debug, DagCborInternal)]
pub struct DecodedSignature {
    #[ipld(default = BTreeMap::new())]
    pub header: BTreeMap<String, Ipld>,
    #[ipld(default = None)]
    pub protected: Option<String>,
    pub signature: String,
}

impl From<EncodedSignature> for DecodedSignature {
    fn from(value: EncodedSignature) -> Self {
        Self {
            header: value.header,
            protected: value.protected.to_base64(),
            signature: value.signature.to_base64(),
        }
    }
}

impl From<Signature> for DecodedSignature {
    fn from(value: Signature) -> Self {
        Self {
            header: value.header,
            protected: value.protected,
            signature: value.signature,
        }
    }
}

/// Decoded form of a JWE recipient
#[derive(PartialEq, Default, Debug, DagCborInternal)]
pub struct DecodedRecipient {
    #[ipld(default = None)]
    pub encrypted_key: Option<String>,
    #[ipld(default = BTreeMap::new())]
    pub header: BTreeMap<String, Ipld>,
}
impl From<EncodedRecipient> for DecodedRecipient {
    fn from(value: EncodedRecipient) -> Self {
        Self {
            encrypted_key: value.encrypted_key.to_base64(),
            header: value.header,
        }
    }
}
impl From<Recipient> for DecodedRecipient {
    fn from(value: Recipient) -> Self {
        Self {
            encrypted_key: value.encrypted_key,
            header: value.header,
        }
    }
}

trait FromBase64<T>: Sized {
    type Error;

    /// Decode a value from base64
    fn from_base64(value: T) -> Result<Self, Self::Error>;
}

impl FromBase64<String> for Bytes {
    type Error = Error;

    fn from_base64(value: String) -> Result<Self, Self::Error> {
        Ok(base64_url::decode(value.as_str())?.into_boxed_slice())
    }
}

impl<T, U> FromBase64<Option<T>> for Option<U>
where
    U: FromBase64<T>,
{
    type Error = U::Error;

    fn from_base64(value: Option<T>) -> Result<Self, Self::Error> {
        value.map(|v| U::from_base64(v)).transpose()
    }
}

trait ToBase64<T>: Sized {
    /// Encode value to base64
    fn to_base64(self) -> T;
}
impl ToBase64<String> for Bytes {
    fn to_base64(self) -> String {
        base64_url::encode(self.as_ref())
    }
}
impl<T, U> ToBase64<Option<T>> for Option<U>
where
    U: ToBase64<T>,
{
    fn to_base64(self) -> Option<T> {
        self.map(|v| v.to_base64())
    }
}
