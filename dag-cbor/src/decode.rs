//! CBOR decoder
use crate::error::{InvalidCidPrefix, LengthOutOfRange, UnexpectedCode, UnexpectedKey, UnknownTag};
use crate::DagCborCodec as DagCbor;
use byteorder::{BigEndian, ByteOrder};
use core::convert::TryFrom;
use libipld_core::cid::Cid;
use libipld_core::codec::{Decode, References};
use libipld_core::error::Result;
use libipld_core::ipld::Ipld;
use libipld_core::link::Link;
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::str::FromStr;
use std::sync::Arc;

/// Reads a u8 from a byte stream.
pub fn read_u8<R: Read>(r: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Reads a u16 from a byte stream.
pub fn read_u16<R: Read>(r: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u16(&buf))
}

/// Reads a u32 from a byte stream.
pub fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u32(&buf))
}

/// Reads a u64 from a byte stream.
pub fn read_u64<R: Read>(r: &mut R) -> Result<u64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u64(&buf))
}

/// Reads a f32 from a byte stream.
pub fn read_f32<R: Read>(r: &mut R) -> Result<f32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_f32(&buf))
}

/// Reads a f64 from a byte stream.
pub fn read_f64<R: Read>(r: &mut R) -> Result<f64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_f64(&buf))
}

/// Reads `len` number of bytes from a byte stream.
pub fn read_bytes<R: Read>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// Reads `len` number of bytes from a byte stream and converts them to a string.
pub fn read_str<R: Read>(r: &mut R, len: usize) -> Result<String> {
    let bytes = read_bytes(r, len)?;
    let string = std::str::from_utf8(&bytes)?;
    Ok(string.to_string())
}

/// Reads bytes from a byte stream and matches them with the key. If the key
/// doesn't match the read bytes it returns an `UnexpectedKey` error.
pub fn read_key<R: Read>(r: &mut R, key: &str) -> Result<()> {
    let key_bytes = key.as_bytes();
    let bytes = read_bytes(r, key.len() + 1)?;
    if key_bytes == &bytes[1..] {
        Ok(())
    } else {
        Err(UnexpectedKey.into())
    }
}

/// Reads any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read<R: Read, T: TryReadCbor>(r: &mut R) -> Result<T> {
    let major = crate::decode::read_u8(r)?;
    if let Some(res) = T::try_read_cbor(r, major)? {
        Ok(res)
    } else {
        Err(UnexpectedCode.into())
    }
}

/// Reads a list of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_list<R: Read, T: TryReadCbor>(r: &mut R, len: usize) -> Result<Vec<T>> {
    let mut list: Vec<T> = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(read(r)?);
    }
    Ok(list)
}

/// Reads a map of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_map<R: Read, K, T: TryReadCbor>(r: &mut R, len: usize) -> Result<BTreeMap<K, T>>
where
    K: FromStr + Ord,
    <K as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let mut map: BTreeMap<K, T> = BTreeMap::new();
    for _ in 0..len {
        let key: String = read(r)?;
        let value = read(r)?;
        map.insert(key.parse()?, value);
    }
    Ok(map)
}

/// Reads a cid from a stream of cbor encoded bytes.
pub fn read_link<R: Read>(r: &mut R) -> Result<Cid> {
    let tag = read_u8(r)?;
    if tag != 42 {
        return Err(UnknownTag.into());
    }
    let ty = read_u8(r)?;
    if ty != 0x58 {
        return Err(UnknownTag.into());
    }
    let len = read_u8(r)?;
    if len == 0 {
        return Err(LengthOutOfRange.into());
    }
    let bytes = read_bytes(r, len as usize)?;
    if bytes[0] != 0 {
        return Err(InvalidCidPrefix(bytes[0]).into());
    }

    // skip the first byte per
    // https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md#links
    Ok(Cid::try_from(&bytes[1..])?)
}

/// Reads the len given a base.
pub fn read_len<R: Read>(r: &mut R, major: u8) -> Result<usize> {
    Ok(match major {
        0x00..=0x17 => major as usize,
        0x18 => read_u8(r)? as usize,
        0x19 => read_u16(r)? as usize,
        0x1a => read_u32(r)? as usize,
        0x1b => {
            let len = read_u64(r)?;
            if len > usize::max_value() as u64 {
                return Err(LengthOutOfRange.into());
            }
            len as usize
        }
        _ => return Err(UnexpectedCode.into()),
    })
}

/// `TryReadCbor` trait.
pub trait TryReadCbor: Sized {
    /// Takes the read major code and tries to parse from a byte stream. If parsing fails the
    /// byte stream must not be advanced.
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>>;
}

macro_rules! impl_decode {
    ($ty:ident) => {
        impl Decode<DagCbor> for $ty {
            fn decode<R: Read>(_: DagCbor, r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
    ($ty:ident<T>) => {
        impl<T: TryReadCbor> Decode<DagCbor> for $ty<T> {
            fn decode<R: Read>(_: DagCbor, r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
    ($ty:ident<$param:ident, T>) => {
        impl<$param, T: TryReadCbor> Decode<DagCbor> for $ty<$param, T>
        where
            $param: FromStr + Ord,
            <$param as FromStr>::Err: std::error::Error + Send + Sync + 'static,
        {
            fn decode<R: Read>(_: DagCbor, r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
    ($ty:ident<[u8]>) => {
        impl Decode<DagCbor> for $ty<[u8]> {
            fn decode<R: Read>(_: DagCbor, r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
}

impl TryReadCbor for bool {
    fn try_read_cbor<R: Read>(_: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xf4 => Ok(Some(false)),
            0xf5 => Ok(Some(true)),
            _ => Ok(None),
        }
    }
}
impl_decode!(bool);

impl TryReadCbor for u8 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major)),
            0x18 => Ok(Some(read_u8(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u8);

impl TryReadCbor for u16 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u16)),
            0x18 => Ok(Some(read_u8(r)? as u16)),
            0x19 => Ok(Some(read_u16(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u16);

impl TryReadCbor for u32 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u32)),
            0x18 => Ok(Some(read_u8(r)? as u32)),
            0x19 => Ok(Some(read_u16(r)? as u32)),
            0x1a => Ok(Some(read_u32(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u32);

impl TryReadCbor for u64 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u64)),
            0x18 => Ok(Some(read_u8(r)? as u64)),
            0x19 => Ok(Some(read_u16(r)? as u64)),
            0x1a => Ok(Some(read_u32(r)? as u64)),
            0x1b => Ok(Some(read_u64(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u64);

impl TryReadCbor for i8 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i8)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i8)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i8);

impl TryReadCbor for i16 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i16)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i16)),
            0x39 => Ok(Some(-1 - read_u16(r)? as i16)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i16);

impl TryReadCbor for i32 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i32)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i32)),
            0x39 => Ok(Some(-1 - read_u16(r)? as i32)),
            0x3a => Ok(Some(-1 - read_u32(r)? as i32)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i32);

impl TryReadCbor for i64 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i64)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i64)),
            0x39 => Ok(Some(-1 - read_u16(r)? as i64)),
            0x3a => Ok(Some(-1 - read_u32(r)? as i64)),
            0x3b => Ok(Some(-1 - read_u64(r)? as i64)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i64);

impl TryReadCbor for f32 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xfa => Ok(Some(read_f32(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(f32);

impl TryReadCbor for f64 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xfa => Ok(Some(read_f32(r)? as f64)),
            0xfb => Ok(Some(read_f64(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(f64);

impl TryReadCbor for String {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x60..=0x7b => {
                let len = read_len(r, major - 0x60)?;
                Ok(Some(read_str(r, len)?))
            }
            _ => Ok(None),
        }
    }
}
impl_decode!(String);

impl TryReadCbor for Cid {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xd8 => Ok(Some(read_link(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(Cid);

impl<T> TryReadCbor for Link<T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        Ok(Cid::try_read_cbor(r, major)?.map(Into::into))
    }
}

impl TryReadCbor for Box<[u8]> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x40..=0x5b => {
                let len = read_len(r, major - 0x40)?;
                Ok(Some(read_bytes(r, len)?.into_boxed_slice()))
            }
            _ => Ok(None),
        }
    }
}
impl_decode!(Box<[u8]>);

impl<T: TryReadCbor> TryReadCbor for Option<T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xf6 => Ok(Some(None)),
            0xf7 => Ok(Some(None)),
            _ => {
                if let Some(res) = T::try_read_cbor(r, major)? {
                    Ok(Some(Some(res)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
impl_decode!(Option<T>);

impl<T: TryReadCbor> TryReadCbor for Vec<T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x80..=0x9b => {
                let len = read_len(r, major - 0x80)?;
                Ok(Some(read_list(r, len)?))
            }
            _ => Ok(None),
        }
    }
}
impl_decode!(Vec<T>);

impl<K, T: TryReadCbor> TryReadCbor for BTreeMap<K, T>
where
    K: FromStr + Ord,
    <K as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xa0..=0xbb => {
                let len = read_len(r, major - 0xa0)?;
                Ok(Some(read_map(r, len)?))
            }
            _ => Ok(None),
        }
    }
}
impl_decode!(BTreeMap<K, T>);

impl TryReadCbor for Ipld {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let ipld = match major {
            // Major type 0: an unsigned integer
            0x00..=0x17 => Self::Integer(major as i128),
            0x18 => Self::Integer(read_u8(r)? as i128),
            0x19 => Self::Integer(read_u16(r)? as i128),
            0x1a => Self::Integer(read_u32(r)? as i128),
            0x1b => Self::Integer(read_u64(r)? as i128),

            // Major type 1: a negative integer
            0x20..=0x37 => Self::Integer(-1 - (major - 0x20) as i128),
            0x38 => Self::Integer(-1 - read_u8(r)? as i128),
            0x39 => Self::Integer(-1 - read_u16(r)? as i128),
            0x3a => Self::Integer(-1 - read_u32(r)? as i128),
            0x3b => Self::Integer(-1 - read_u64(r)? as i128),

            // Major type 2: a byte string
            0x40..=0x5b => {
                let len = read_len(r, major - 0x40)?;
                let bytes = read_bytes(r, len as usize)?;
                Self::Bytes(bytes)
            }

            // Major type 3: a text string
            0x60..=0x7b => {
                let len = read_len(r, major - 0x60)?;
                let string = read_str(r, len as usize)?;
                Self::String(string)
            }

            // Major type 4: an array of data items
            0x80..=0x9b => {
                let len = read_len(r, major - 0x80)?;
                let list = read_list(r, len as usize)?;
                Self::List(list)
            }

            // Major type 5: a map of pairs of data items
            0xa0..=0xbb => {
                let len = read_len(r, major - 0xa0)?;
                let map = read_map(r, len as usize)?;
                Self::Map(map)
            }

            // Major type 6: optional semantic tagging of other major types
            0xd8 => Self::Link(read_link(r)?),

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4 => Self::Bool(false),
            0xf5 => Self::Bool(true),
            0xf6 => Self::Null,
            0xf7 => Self::Null,
            0xfa => Self::Float(read_f32(r)? as f64),
            0xfb => Self::Float(read_f64(r)?),
            _ => return Ok(None),
        };
        Ok(Some(ipld))
    }
}
impl_decode!(Ipld);
impl References<DagCbor> for Ipld {
    fn references<R: Read + Seek, E: Extend<Cid>>(
        c: DagCbor,
        r: &mut R,
        set: &mut E,
    ) -> Result<()> {
        let major = read_u8(r)?;
        match major {
            // Major type 0: an unsigned integer
            0x00..=0x17 => {}
            0x18 => {
                r.seek(SeekFrom::Current(1))?;
            }
            0x19 => {
                r.seek(SeekFrom::Current(2))?;
            }
            0x1a => {
                r.seek(SeekFrom::Current(4))?;
            }
            0x1b => {
                r.seek(SeekFrom::Current(8))?;
            }

            // Major type 1: a negative integer
            0x20..=0x37 => {}
            0x38 => {
                r.seek(SeekFrom::Current(1))?;
            }
            0x39 => {
                r.seek(SeekFrom::Current(2))?;
            }
            0x3a => {
                r.seek(SeekFrom::Current(4))?;
            }
            0x3b => {
                r.seek(SeekFrom::Current(8))?;
            }

            // Major type 2: a byte string
            0x40..=0x5b => {
                let len = read_len(r, major - 0x40)?;
                r.seek(SeekFrom::Current(len as _))?;
            }

            // Major type 3: a text string
            0x60..=0x7b => {
                let len = read_len(r, major - 0x60)?;
                r.seek(SeekFrom::Current(len as _))?;
            }

            // Major type 4: an array of data items
            0x80..=0x9b => {
                let len = read_len(r, major - 0x80)?;
                for _ in 0..len {
                    <Self as References<DagCbor>>::references(c, r, set)?;
                }
            }

            // Major type 5: a map of pairs of data items
            0xa0..=0xb7 => {
                let len = read_len(r, major - 0xa0)?;
                for _ in 0..len {
                    <Self as References<DagCbor>>::references(c, r, set)?;
                    <Self as References<DagCbor>>::references(c, r, set)?;
                }
            }

            // Major type 6: optional semantic tagging of other major types
            0xd8 => {
                set.extend(std::iter::once(read_link(r)?));
            }

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4..=0xf7 => {}
            0xfa => {
                r.seek(SeekFrom::Current(4))?;
            }
            0xfb => {
                r.seek(SeekFrom::Current(8))?;
            }
            _ => return Err(UnexpectedCode.into()),
        };
        Ok(())
    }
}

impl<T: TryReadCbor> TryReadCbor for Arc<T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        if let Some(res) = T::try_read_cbor(r, major)? {
            Ok(Some(Arc::new(res)))
        } else {
            Ok(None)
        }
    }
}
impl_decode!(Arc<T>);
