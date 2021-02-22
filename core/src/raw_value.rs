//! misc stuff
use std::{
    io::{self, Read, Seek},
    marker::PhantomData,
};

use crate::codec::{Codec, Decode, Encode};
use std::convert::TryFrom;

/// A raw value for a certain codec.
///
/// Contains the raw, unprocessed data for a single item for that particular codec
pub struct RawValue<C> {
    data: Box<[u8]>,
    _p: PhantomData<C>,
}

impl<C> RawValue<C> {
    fn new(data: Box<[u8]>) -> Self {
        Self {
            data,
            _p: PhantomData,
        }
    }
}

/// trait to implement to skip a single item at the current position
pub trait SkipOne: Codec {
    /// assuming r is at the start of an item, advance r to the end
    fn skip<R: Read + Seek>(&self, r: &mut R) -> anyhow::Result<()>;
}

impl<C: Codec + SkipOne> Decode<C> for RawValue<C> {
    fn decode<R: std::io::Read + std::io::Seek>(c: C, r: &mut R) -> anyhow::Result<Self> {
        let p0 = r.seek(io::SeekFrom::Current(0))?;
        c.skip(r)?;
        let p1 = r.seek(io::SeekFrom::Current(0))?;
        anyhow::ensure!(p1 > p0);
        let len = usize::try_from(p1 - p0)?;
        r.seek(io::SeekFrom::Start(p0))?;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        Ok(Self::new(buf.into()))
    }
}

impl<C: Codec> Encode<C> for RawValue<C> {
    fn encode<W: std::io::Write>(&self, _: C, w: &mut W) -> anyhow::Result<()> {
        w.write_all(&self.data)?;
        Ok(())
    }
}

/// Allows to ignore a single item
pub struct IgnoredAny;

impl<C: Codec + SkipOne> Decode<C> for IgnoredAny {
    fn decode<R: std::io::Read + std::io::Seek>(c: C, r: &mut R) -> anyhow::Result<Self> {
        c.skip(r)?;
        Ok(Self)
    }
}
