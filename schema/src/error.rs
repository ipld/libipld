use crate::dev::*;
use cid::Error as CidError;
use failure::Fail;
use libipld_base::error::{BlockError, IpldError};
use std::{convert::Infallible, num::TryFromIntError, string::FromUtf8Error};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Block error: {}", _0)]
    Block(BlockError),

    #[fail(display = "Cid error: {}", _0)]
    Cid(CidError),

    #[fail(display = "IPLD Codec error: {}", _0)]
    Codec(failure::Error),

    #[fail(display = "Invalid data received from context: {}", _0)]
    Context(failure::Error),

    #[fail(display = "Ipld error: {}", _0)]
    Ipld(IpldError),
}

impl From<Error> for BlockError {
    fn from(err: Error) -> Self {
        match err {
            Error::Block(err) => err,
            Error::Cid(err) => BlockError::Cid(err),
            err => BlockError::CodecError(err.into()),
        }
    }
}

impl From<BlockError> for Error {
    fn from(err: BlockError) -> Self {
        Error::Block(err)
    }
}

impl From<CidError> for Error {
    fn from(err: CidError) -> Self {
        Error::Cid(err)
    }
}

impl From<IpldError> for Error {
    fn from(err: IpldError) -> Self {
        Error::Ipld(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::Codec(err.into())
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Error::Codec(err.into())
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        Error::Codec(err.into())
    }
}
