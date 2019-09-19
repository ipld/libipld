//! CBOR decoder
#![allow(missing_docs)]
use crate::error::Result;
use crate::ipld::Ipld;
use async_std::io::Read as _;
use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use cid::Cid;
use core::convert::TryFrom;
use failure::Fail;
pub use futures_io::AsyncRead as Read;
use std::collections::BTreeMap;

#[derive(Debug, Fail)]
pub enum CborError {
    #[fail(display = "Length out of range.")]
    LengthOutOfRange,
    #[fail(display = "Unexpected code.")]
    UnexpectedCode,
    #[fail(display = "Unkown tag.")]
    UnknownTag,
    #[fail(display = "Wrong key.")]
    UnexpectedKey,
    #[fail(display = "{}", _0)]
    Io(std::io::Error),
}

impl From<std::io::Error> for CborError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[inline]
pub async fn read_u8<R: Read + Unpin + Send>(r: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    r.read_exact(&mut buf).await?;
    Ok(buf[0])
}

#[inline]
pub async fn read_u16<R: Read + Unpin + Send>(r: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    r.read_exact(&mut buf).await?;
    Ok(BigEndian::read_u16(&buf))
}

#[inline]
pub async fn read_u32<R: Read + Unpin + Send>(r: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf).await?;
    Ok(BigEndian::read_u32(&buf))
}

#[inline]
pub async fn read_u64<R: Read + Unpin + Send>(r: &mut R) -> Result<u64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf).await?;
    Ok(BigEndian::read_u64(&buf))
}

#[inline]
pub async fn read_f32<R: Read + Unpin + Send>(r: &mut R) -> Result<f32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf).await?;
    Ok(BigEndian::read_f32(&buf))
}

#[inline]
pub async fn read_f64<R: Read + Unpin + Send>(r: &mut R) -> Result<f64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf).await?;
    Ok(BigEndian::read_f64(&buf))
}

#[inline]
pub async fn read_bytes<R: Read + Unpin + Send>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0; len];
    r.read_exact(&mut buf).await?;
    Ok(buf)
}

#[inline]
pub async fn read_str<R: Read + Unpin + Send>(r: &mut R, len: usize) -> Result<String> {
    let bytes = read_bytes(r, len).await?;
    let string = std::str::from_utf8(&bytes)?;
    Ok(string.to_string())
}

#[inline]
pub async fn read_key<R: Read + Unpin + Send>(r: &mut R, key: &str) -> Result<()> {
    let key_bytes = key.as_bytes();
    let bytes = read_bytes(r, key.len() + 1).await?;
    if key_bytes == &bytes[1..] {
        Ok(())
    } else {
        Err(CborError::UnexpectedKey.into())
    }
}

#[inline]
pub async fn read_list<R: Read + Unpin + Send, T: ReadCbor + Send>(
    r: &mut R,
    len: usize,
) -> Result<Vec<T>> {
    let mut list: Vec<T> = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(T::read_cbor(r).await?);
    }
    Ok(list)
}

#[inline]
pub async fn read_map<R: Read + Unpin + Send, T: ReadCbor + Send>(
    r: &mut R,
    len: usize,
) -> Result<BTreeMap<String, T>> {
    let mut map: BTreeMap<String, T> = BTreeMap::new();
    for _ in 0..len {
        let key = String::read_cbor(r).await?;
        let value = T::read_cbor(r).await?;
        map.insert(key, value);
    }
    Ok(map)
}

#[inline]
pub async fn read_link<R: Read + Unpin + Send>(r: &mut R) -> Result<Cid> {
    let tag = read_u8(r).await?;
    if tag != 42 {
        return Err(CborError::UnknownTag.into());
    }
    let ty = read_u8(r).await?;
    if ty != 0x58 {
        return Err(CborError::UnknownTag.into());
    }
    let len = read_u8(r).await?;
    let bytes = read_bytes(r, len as usize).await?;
    Ok(Cid::try_from(bytes)?)
}

#[async_trait]
pub trait ReadCbor: Sized {
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>>;

    #[inline]
    async fn read_cbor<R: Read + Unpin + Send>(r: &mut R) -> Result<Self> {
        let major = read_u8(r).await?;
        if let Some(res) = Self::try_read_cbor(r, major).await? {
            Ok(res)
        } else {
            Err(CborError::UnexpectedCode.into())
        }
    }
}

#[async_trait]
impl ReadCbor for bool {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(_: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xf4 => Ok(Some(false)),
            0xf5 => Ok(Some(true)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for u8 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major)),
            0x18 => Ok(Some(read_u8(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for u16 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u16)),
            0x18 => Ok(Some(read_u8(r).await? as u16)),
            0x19 => Ok(Some(read_u16(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for u32 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u32)),
            0x18 => Ok(Some(read_u8(r).await? as u32)),
            0x19 => Ok(Some(read_u16(r).await? as u32)),
            0x1a => Ok(Some(read_u32(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for u64 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u64)),
            0x18 => Ok(Some(read_u8(r).await? as u64)),
            0x19 => Ok(Some(read_u16(r).await? as u64)),
            0x1a => Ok(Some(read_u32(r).await? as u64)),
            0x1b => Ok(Some(read_u64(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for i8 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i8)),
            0x38 => Ok(Some(-1 - read_u8(r).await? as i8)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for i16 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i16)),
            0x38 => Ok(Some(-1 - read_u8(r).await? as i16)),
            0x39 => Ok(Some(-1 - read_u16(r).await? as i16)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for i32 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i32)),
            0x38 => Ok(Some(-1 - read_u8(r).await? as i32)),
            0x39 => Ok(Some(-1 - read_u16(r).await? as i32)),
            0x3a => Ok(Some(-1 - read_u32(r).await? as i32)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for i64 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i64)),
            0x38 => Ok(Some(-1 - read_u8(r).await? as i64)),
            0x39 => Ok(Some(-1 - read_u16(r).await? as i64)),
            0x3a => Ok(Some(-1 - read_u32(r).await? as i64)),
            0x3b => Ok(Some(-1 - read_u64(r).await? as i64)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for f32 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xfa => Ok(Some(read_f32(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for f64 {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xfa => Ok(Some(read_f32(r).await? as f64)),
            0xfb => Ok(Some(read_f64(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for String {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0x60..=0x77 => major as usize - 0x60,
            0x78 => read_u8(r).await? as usize,
            0x79 => read_u16(r).await? as usize,
            0x7a => read_u32(r).await? as usize,
            0x7b => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_str(r, len).await?))
    }
}

#[async_trait]
impl ReadCbor for Cid {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xd8 => Ok(Some(read_link(r).await?)),
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl ReadCbor for Box<[u8]> {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0x40..=0x57 => major as usize - 0x40,
            0x58 => read_u8(r).await? as usize,
            0x59 => read_u16(r).await? as usize,
            0x5a => read_u32(r).await? as usize,
            0x5b => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_bytes(r, len).await?.into_boxed_slice()))
    }
}

#[async_trait]
impl<T: ReadCbor> ReadCbor for Option<T> {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xf6 => Ok(Some(None)),
            0xf7 => Ok(Some(None)),
            _ => {
                if let Some(res) = T::try_read_cbor(r, major).await? {
                    Ok(Some(Some(res)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[async_trait]
impl<T: ReadCbor + Send> ReadCbor for Vec<T> {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0x80..=0x97 => major as usize - 0x80,
            0x98 => read_u8(r).await? as usize,
            0x99 => read_u16(r).await? as usize,
            0x9a => read_u32(r).await? as usize,
            0x9b => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_list(r, len).await?))
    }
}

#[async_trait]
impl<T: ReadCbor + Send> ReadCbor for BTreeMap<String, T> {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0xa0..=0xb7 => major as usize - 0xa0,
            0xb8 => read_u8(r).await? as usize,
            0xb9 => read_u16(r).await? as usize,
            0xba => read_u32(r).await? as usize,
            0xbb => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_map(r, len).await?))
    }
}

#[async_trait]
impl ReadCbor for Ipld {
    #[inline]
    async fn try_read_cbor<R: Read + Unpin + Send>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let ipld = match major {
            // Major type 0: an unsigned integer
            0x00..=0x17 => Ipld::Integer(major as i128),
            0x18 => Ipld::Integer(read_u8(r).await? as i128),
            0x19 => Ipld::Integer(read_u16(r).await? as i128),
            0x1a => Ipld::Integer(read_u32(r).await? as i128),
            0x1b => Ipld::Integer(read_u64(r).await? as i128),

            // Major type 1: a negative integer
            0x20..=0x37 => Ipld::Integer(-1 - (major - 0x20) as i128),
            0x38 => Ipld::Integer(-1 - read_u8(r).await? as i128),
            0x39 => Ipld::Integer(-1 - read_u16(r).await? as i128),
            0x3a => Ipld::Integer(-1 - read_u32(r).await? as i128),
            0x3b => Ipld::Integer(-1 - read_u64(r).await? as i128),

            // Major type 2: a byte string
            0x40..=0x57 => {
                let len = major - 0x40;
                let bytes = read_bytes(r, len as usize).await?;
                Ipld::Bytes(bytes)
            }
            0x58 => {
                let len = read_u8(r).await?;
                let bytes = read_bytes(r, len as usize).await?;
                Ipld::Bytes(bytes)
            }
            0x59 => {
                let len = read_u16(r).await?;
                let bytes = read_bytes(r, len as usize).await?;
                Ipld::Bytes(bytes)
            }
            0x5a => {
                let len = read_u32(r).await?;
                let bytes = read_bytes(r, len as usize).await?;
                Ipld::Bytes(bytes)
            }
            0x5b => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let bytes = read_bytes(r, len as usize).await?;
                Ipld::Bytes(bytes)
            }

            // Major type 3: a text string
            0x60..=0x77 => {
                let len = major - 0x60;
                let string = read_str(r, len as usize).await?;
                Ipld::String(string)
            }
            0x78 => {
                let len = read_u8(r).await?;
                let string = read_str(r, len as usize).await?;
                Ipld::String(string)
            }
            0x79 => {
                let len = read_u16(r).await?;
                let string = read_str(r, len as usize).await?;
                Ipld::String(string)
            }
            0x7a => {
                let len = read_u32(r).await?;
                let string = read_str(r, len as usize).await?;
                Ipld::String(string)
            }
            0x7b => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let string = read_str(r, len as usize).await?;
                Ipld::String(string)
            }

            // Major type 4: an array of data items
            0x80..=0x97 => {
                let len = major - 0x80;
                let list = read_list(r, len as usize).await?;
                Ipld::List(list)
            }
            0x98 => {
                let len = read_u8(r).await?;
                let list = read_list(r, len as usize).await?;
                Ipld::List(list)
            }
            0x99 => {
                let len = read_u16(r).await?;
                let list = read_list(r, len as usize).await?;
                Ipld::List(list)
            }
            0x9a => {
                let len = read_u32(r).await?;
                let list = read_list(r, len as usize).await?;
                Ipld::List(list)
            }
            0x9b => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let list = read_list(r, len as usize).await?;
                Ipld::List(list)
            }

            // Major type 5: a map of pairs of data items
            0xa0..=0xb7 => {
                let len = major - 0xa0;
                let map = read_map(r, len as usize).await?;
                Ipld::Map(map)
            }
            0xb8 => {
                let len = read_u8(r).await?;
                let map = read_map(r, len as usize).await?;
                Ipld::Map(map)
            }
            0xb9 => {
                let len = read_u16(r).await?;
                let map = read_map(r, len as usize).await?;
                Ipld::Map(map)
            }
            0xba => {
                let len = read_u32(r).await?;
                let map = read_map(r, len as usize).await?;
                Ipld::Map(map)
            }
            0xbb => {
                let len = read_u64(r).await?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let map = read_map(r, len as usize).await?;
                Ipld::Map(map)
            }

            // Major type 6: optional semantic tagging of other major types
            0xd8 => Ipld::Link(read_link(r).await?),

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4 => Ipld::Bool(false),
            0xf5 => Ipld::Bool(true),
            0xf6 => Ipld::Null,
            0xf7 => Ipld::Null,
            0xfa => Ipld::Float(read_f32(r).await? as f64),
            0xfb => Ipld::Float(read_f64(r).await?),
            _ => return Ok(None),
        };
        Ok(Some(ipld))
    }
}
