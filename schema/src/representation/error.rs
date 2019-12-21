use crate::{BlockError, CborError, CidError, IpldError};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Block error: {}", _0)]
    Block(BlockError),

    #[fail(display = "Cbor error: {}", _0)]
    Cbor(CborError),

    #[fail(display = "Cid error: {}", _0)]
    Cid(CidError),

    #[fail(display = "Ipld error: {}", _0)]
    Ipld(IpldError),
}

impl From<BlockError> for Error {
    fn from(err: BlockError) -> Self {
        Error::Block(err)
    }
}

impl From<CborError> for Error {
    fn from(err: CborError) -> Self {
        Error::Cbor(err)
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
