//! CBOR decoder
use crate::error::{InvalidCidPrefix, LengthOutOfRange, UnexpectedCode, UnknownTag};
use crate::DagCborCodec as DagCbor;
use byteorder::{BigEndian, ByteOrder};
use core::convert::TryFrom;
use libipld_core::codec::{Decode, References};
use libipld_core::error::Result;
use libipld_core::ipld::Ipld;
use libipld_core::{cid::Cid, raw_value::SkipOne};
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

/// Reads a u8 from a byte stream.
pub fn read_u8<R: Read + Seek>(r: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Reads a u16 from a byte stream.
pub fn read_u16<R: Read + Seek>(r: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u16(&buf))
}

/// Reads a u32 from a byte stream.
pub fn read_u32<R: Read + Seek>(r: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u32(&buf))
}

/// Reads a u64 from a byte stream.
pub fn read_u64<R: Read + Seek>(r: &mut R) -> Result<u64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u64(&buf))
}

/// Reads a f32 from a byte stream.
pub fn read_f32<R: Read + Seek>(r: &mut R) -> Result<f32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_f32(&buf))
}

/// Reads a f64 from a byte stream.
pub fn read_f64<R: Read + Seek>(r: &mut R) -> Result<f64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_f64(&buf))
}

/// Reads `len` number of bytes from a byte stream.
pub fn read_bytes<R: Read + Seek>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// Reads `len` number of bytes from a byte stream and converts them to a string.
pub fn read_str<R: Read + Seek>(r: &mut R, len: usize) -> Result<String> {
    let bytes = read_bytes(r, len)?;
    Ok(String::from_utf8(bytes)?)
}

/// Reads a list of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_list<R: Read + Seek, T: Decode<DagCbor>>(r: &mut R, len: usize) -> Result<Vec<T>> {
    let mut list: Vec<T> = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(T::decode(DagCbor, r)?);
    }
    Ok(list)
}

/// Reads a list of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_list_il<R: Read + Seek, T: Decode<DagCbor>>(r: &mut R) -> Result<Vec<T>> {
    let mut list: Vec<T> = Vec::new();
    loop {
        let major = read_u8(r)?;
        if major == 0xff {
            break;
        }
        r.seek(SeekFrom::Current(-1))?;
        let value = T::decode(DagCbor, r)?;
        list.push(value);
    }
    Ok(list)
}

/// Reads a map of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_map<R: Read + Seek, K: Decode<DagCbor> + Ord, T: Decode<DagCbor>>(
    r: &mut R,
    len: usize,
) -> Result<BTreeMap<K, T>> {
    let mut map: BTreeMap<K, T> = BTreeMap::new();
    for _ in 0..len {
        let key = K::decode(DagCbor, r)?;
        let value = T::decode(DagCbor, r)?;
        map.insert(key, value);
    }
    Ok(map)
}

/// Reads a map of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_map_il<R: Read + Seek, K: Decode<DagCbor> + Ord, T: Decode<DagCbor>>(
    r: &mut R,
) -> Result<BTreeMap<K, T>> {
    let mut map: BTreeMap<K, T> = BTreeMap::new();
    loop {
        let major = read_u8(r)?;
        if major == 0xff {
            break;
        }
        r.seek(SeekFrom::Current(-1))?;
        let key = K::decode(DagCbor, r)?;
        let value = T::decode(DagCbor, r)?;
        map.insert(key, value);
    }
    Ok(map)
}

/// Reads a cid from a stream of cbor encoded bytes.
pub fn read_link<R: Read + Seek>(r: &mut R) -> Result<Cid> {
    let ty = read_u8(r)?;
    if ty != 0x58 {
        return Err(UnknownTag(ty).into());
    }
    let len = read_u8(r)?;
    if len == 0 {
        return Err(LengthOutOfRange::new::<Cid>().into());
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
pub fn read_len<R: Read + Seek>(r: &mut R, major: u8) -> Result<usize> {
    Ok(match major {
        0x00..=0x17 => major as usize,
        0x18 => read_u8(r)? as usize,
        0x19 => read_u16(r)? as usize,
        0x1a => read_u32(r)? as usize,
        0x1b => {
            let len = read_u64(r)?;
            if len > usize::max_value() as u64 {
                return Err(LengthOutOfRange::new::<usize>().into());
            }
            len as usize
        }
        major => return Err(UnexpectedCode::new::<usize>(major).into()),
    })
}

impl Decode<DagCbor> for bool {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0xf4 => false,
            0xf5 => true,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for u8 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x00..=0x17 => major,
            0x18 => read_u8(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for u16 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x00..=0x17 => major as u16,
            0x18 => read_u8(r)? as u16,
            0x19 => read_u16(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for u32 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x00..=0x17 => major as u32,
            0x18 => read_u8(r)? as u32,
            0x19 => read_u16(r)? as u32,
            0x1a => read_u32(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for u64 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x00..=0x17 => major as u64,
            0x18 => read_u8(r)? as u64,
            0x19 => read_u16(r)? as u64,
            0x1a => read_u32(r)? as u64,
            0x1b => read_u64(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for i8 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x20..=0x37 => -1 - (major - 0x20) as i8,
            0x38 => -1 - read_u8(r)? as i8,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for i16 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x20..=0x37 => -1 - (major - 0x20) as i16,
            0x38 => -1 - read_u8(r)? as i16,
            0x39 => -1 - read_u16(r)? as i16,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for i32 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x20..=0x37 => -1 - (major - 0x20) as i32,
            0x38 => -1 - read_u8(r)? as i32,
            0x39 => -1 - read_u16(r)? as i32,
            0x3a => -1 - read_u32(r)? as i32,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for i64 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x20..=0x37 => -1 - (major - 0x20) as i64,
            0x38 => -1 - read_u8(r)? as i64,
            0x39 => -1 - read_u16(r)? as i64,
            0x3a => -1 - read_u32(r)? as i64,
            0x3b => -1 - read_u64(r)? as i64,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for f32 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0xfa => read_f32(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for f64 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0xfa => read_f32(r)? as f64,
            0xfb => read_f64(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for String {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x60..=0x7b => {
                let len = read_len(r, major - 0x60)?;
                read_str(r, len)?
            }
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for Cid {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        if major == 0xd8 {
            if let Ok(tag) = read_u8(r) {
                if tag == 42 {
                    return read_link(r);
                }
            }
        }
        Err(UnexpectedCode::new::<Self>(major).into())
    }
}

impl Decode<DagCbor> for Box<[u8]> {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x40..=0x5b => {
                let len = read_len(r, major - 0x40)?;
                read_bytes(r, len)?.into_boxed_slice()
            }
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl<T: Decode<DagCbor>> Decode<DagCbor> for Option<T> {
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0xf6 => None,
            0xf7 => None,
            _ => {
                r.seek(SeekFrom::Current(-1))?;
                Some(T::decode(c, r)?)
            }
        };
        Ok(result)
    }
}

impl<T: Decode<DagCbor>> Decode<DagCbor> for Vec<T> {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x80..=0x9b => {
                let len = read_len(r, major - 0x80)?;
                read_list(r, len)?
            }
            0x9f => read_list_il(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl<K: Decode<DagCbor> + Ord, T: Decode<DagCbor>> Decode<DagCbor> for BTreeMap<K, T> {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0xa0..=0xbb => {
                let len = read_len(r, major - 0xa0)?;
                read_map(r, len)?
            }
            0xbf => read_map_il(r)?,
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl Decode<DagCbor> for Ipld {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
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

            // Major type 4: an array of data items (indefinite length)
            0x9f => {
                let list = read_list_il(r)?;
                Self::List(list)
            }

            // Major type 5: a map of pairs of data items
            0xa0..=0xbb => {
                let len = read_len(r, major - 0xa0)?;
                #[cfg(feature = "unleashed")]
                if len > 0 {
                    let pos = r.seek(SeekFrom::Current(0))?;
                    if let Ok(map) = read_map(r, len as usize) {
                        return Ok(Self::IntegerMap(map));
                    }
                    r.seek(SeekFrom::Start(pos))?;
                }
                Self::StringMap(read_map(r, len as usize)?)
            }

            // Major type 5: a map of pairs of data items (indefinite length)
            0xbf => {
                let pos = r.seek(SeekFrom::Current(0))?;
                #[cfg(feature = "unleashed")]
                if let Ok(map) = read_map_il(r) {
                    return Ok(Self::IntegerMap(map));
                }
                r.seek(SeekFrom::Start(pos))?;
                Self::StringMap(read_map_il(r)?)
            }

            // Major type 6: optional semantic tagging of other major types
            0xd8 => {
                let tag = read_u8(r)?;
                if tag == 42 {
                    Self::Link(read_link(r)?)
                } else {
                    #[cfg(not(feature = "unleashed"))]
                    return Err(UnknownTag(tag).into());
                    #[cfg(feature = "unleashed")]
                    Self::Tag(tag as _, Box::new(Self::decode(DagCbor, r)?))
                }
            }

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4 => Self::Bool(false),
            0xf5 => Self::Bool(true),
            0xf6 => Self::Null,
            0xf7 => Self::Null,
            0xfa => Self::Float(read_f32(r)? as f64),
            0xfb => Self::Float(read_f64(r)?),
            _ => return Err(UnexpectedCode::new::<Self>(major).into()),
        };
        Ok(ipld)
    }
}

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

            // Major type 4: an array of data items (indefinite length)
            0x9f => loop {
                let major = read_u8(r)?;
                if major == 0xff {
                    break;
                }
                r.seek(SeekFrom::Current(-1))?;
                <Self as References<DagCbor>>::references(c, r, set)?;
            },

            // Major type 5: a map of pairs of data items
            0xa0..=0xbb => {
                let len = read_len(r, major - 0xa0)?;
                for _ in 0..len {
                    <Self as References<DagCbor>>::references(c, r, set)?;
                    <Self as References<DagCbor>>::references(c, r, set)?;
                }
            }

            // Major type 5: a map of pairs of data items (indefinite length)
            0xbf => loop {
                let major = read_u8(r)?;
                if major == 0xff {
                    break;
                }
                r.seek(SeekFrom::Current(-1))?;
                <Self as References<DagCbor>>::references(c, r, set)?;
                <Self as References<DagCbor>>::references(c, r, set)?;
            },

            // Major type 6: optional semantic tagging of other major types
            0xd8 => {
                let tag = read_u8(r)?;
                if tag == 42 {
                    set.extend(std::iter::once(read_link(r)?));
                } else {
                    <Self as References<DagCbor>>::references(c, r, set)?;
                }
            }

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4..=0xf7 => {}
            0xf8 => {
                r.seek(SeekFrom::Current(1))?;
            }
            0xf9 => {
                r.seek(SeekFrom::Current(2))?;
            }
            0xfa => {
                r.seek(SeekFrom::Current(4))?;
            }
            0xfb => {
                r.seek(SeekFrom::Current(8))?;
            }
            major => return Err(UnexpectedCode::new::<Ipld>(major).into()),
        };
        Ok(())
    }
}

impl<T: Decode<DagCbor>> Decode<DagCbor> for Arc<T> {
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        Ok(Arc::new(T::decode(c, r)?))
    }
}

impl Decode<DagCbor> for () {
    fn decode<R: Read + Seek>(_c: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x80 => (),
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl<A: Decode<DagCbor>> Decode<DagCbor> for (A,) {
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x81 => (A::decode(c, r)?,),
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl<A: Decode<DagCbor>, B: Decode<DagCbor>> Decode<DagCbor> for (A, B) {
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x82 => (A::decode(c, r)?, B::decode(c, r)?),
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl<A: Decode<DagCbor>, B: Decode<DagCbor>, C: Decode<DagCbor>> Decode<DagCbor> for (A, B, C) {
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x83 => (A::decode(c, r)?, B::decode(c, r)?, C::decode(c, r)?),
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl<A: Decode<DagCbor>, B: Decode<DagCbor>, C: Decode<DagCbor>, D: Decode<DagCbor>> Decode<DagCbor>
    for (A, B, C, D)
{
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_u8(r)?;
        let result = match major {
            0x84 => (
                A::decode(c, r)?,
                B::decode(c, r)?,
                C::decode(c, r)?,
                D::decode(c, r)?,
            ),
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(result)
    }
}

impl SkipOne for DagCbor {
    fn skip<R: Read + Seek>(&self, r: &mut R) -> Result<()> {
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
                    self.skip(r)?;
                }
            }

            // Major type 4: an array of data items (indefinite length)
            0x9f => loop {
                let major = read_u8(r)?;
                if major == 0xff {
                    break;
                }
                r.seek(SeekFrom::Current(-1))?;
                self.skip(r)?;
            },

            // Major type 5: a map of pairs of data items
            0xa0..=0xbb => {
                let len = read_len(r, major - 0xa0)?;
                for _ in 0..len {
                    self.skip(r)?;
                    self.skip(r)?;
                }
            }

            // Major type 5: a map of pairs of data items (indefinite length)
            0xbf => loop {
                let major = read_u8(r)?;
                if major == 0xff {
                    break;
                }
                r.seek(SeekFrom::Current(-1))?;
                self.skip(r)?;
                self.skip(r)?;
            },

            // Major type 6: optional semantic tagging of other major types
            0xd8 => {
                let _tag = read_u8(r)?;
                self.skip(r)?;
            }

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4..=0xf7 => {}
            0xf8 => {
                r.seek(SeekFrom::Current(1))?;
            }
            0xf9 => {
                r.seek(SeekFrom::Current(2))?;
            }
            0xfa => {
                r.seek(SeekFrom::Current(4))?;
            }
            0xfb => {
                r.seek(SeekFrom::Current(8))?;
            }
            major => return Err(UnexpectedCode::new::<Ipld>(major).into()),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DagCborCodec;
    use libipld_core::codec::Codec;
    use libipld_macro::ipld;

    #[test]
    fn il_map() {
        let bytes = [
            0xBF, // Start indefinite-length map
            0x63, // First key, UTF-8 string length 3
            0x46, 0x75, 0x6e, // "Fun"
            0xF5, // First value, true
            0x63, // Second key, UTF-8 string length 3
            0x41, 0x6d, 0x74, // "Amt"
            0x21, // Second value, -2
            0xFF, // "break"
        ];
        let ipld = ipld!({
            "Fun": true,
            "Amt": -2,
        });
        let ipld2: Ipld = DagCborCodec.decode(&bytes).unwrap();
        assert_eq!(ipld, ipld2);
    }

    #[test]
    fn tuples() -> Result<()> {
        let data = ("hello".to_string(), "world".to_string());
        let bytes = DagCborCodec.encode(&data)?;
        println!("{:x?}", bytes);
        let data2: (String, String) = DagCborCodec.decode(&bytes)?;
        assert_eq!(data, data2);
        Ok(())
    }
}
