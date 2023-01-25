//! JOSE error types.
use base64_url::base64::DecodeError;
use libipld_core::cid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("data not a JWE value")]
    NotJwe,
    #[error("data not a JWE value")]
    NotJws,
    #[error("invalid CID data in payload")]
    InvalidCid(#[from] cid::Error),
    #[error("invalid base64 url data")]
    InvalidBase64Url(#[from] DecodeError),
}
