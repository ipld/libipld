//! CBOR decoder
use crate::error::{
    InvalidCidPrefix, LengthOutOfRange, NumberNotMinimal, NumberOutOfRange, UnexpectedCode,
    UnexpectedEof, UnknownTag,
};
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

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct Major(u8);

impl Major {
    const fn new(kind: MajorKind, info: u8) -> Self {
        Major(((kind as u8) << 5) | info)
    }

    #[inline(always)]
    const fn kind(self) -> MajorKind {
        // This is a 3 bit value, so value 0-7 are covered.
        unsafe { std::mem::transmute(self.0 >> 5) }
    }

    #[inline(always)]
    const fn info(self) -> u8 {
        self.0 & 0x1f
    }

    #[inline(always)]
    const fn len(self) -> u8 {
        // All major types follow the same rules for "additioanl bytes".
        // 24 -> 1, 25 -> 2, 26 -> 4, 27 -> 8
        match self.info() {
            info @ 24..=27 => (1 << info - 24),
            _ => 0,
        }
    }
}

// This is the core of the validation logic. Every major type passes through here giving us a chance
// to determine if it's something we allow.
impl TryFrom<u8> for Major {
    type Error = UnexpectedCode;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // We don't allow any major types with additional info 28-31 inclusive.
        // Or the bitmask 0b00011100 = 28.
        if value & 28 == 28 {
            return Err(UnexpectedCode::new::<Ipld>(value));
        } else if (value >> 5) == MajorKind::Other as u8 {
            match value & 0x1f {
                // False, True, Null. TODO: Allow undefined?
                20 | 21 | 22 => (),
                // Floats. TODO: forbid f16 & f32?
                25 | 26 | 27 => (),
                // Everything is forbidden.
                _ => {
                    return Err(UnexpectedCode::new::<Ipld>(value));
                }
            }
        }
        Ok(Major(value))
    }
}

mod consts {
    use super::Major;
    use super::MajorKind::*;

    pub(super) const FALSE: Major = Major::new(Other, 20);
    pub(super) const TRUE: Major = Major::new(Other, 21);
    pub(super) const NULL: Major = Major::new(Other, 22);
    pub(super) const F32: Major = Major::new(Other, 26);
    pub(super) const F64: Major = Major::new(Other, 27);
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
enum MajorKind {
    UnsignedInt = 0,
    NegativeInt = 1,
    ByteString = 2,
    TextString = 3,
    Array = 4,
    Map = 5,
    Tag = 6,
    Other = 7,
}

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
pub fn read_bytes<R: Read>(r: &mut R, len: u64) -> Result<Vec<u8>> {
    let len = usize::try_from(len).map_err(|_| LengthOutOfRange::new::<usize>())?;
    // Limit up-front allocations to 16KiB as the length is user controlled.
    let mut buf = Vec::with_capacity(len.min(16 * 1024));
    r.take(len as u64).read_to_end(&mut buf)?;
    if buf.len() != len {
        return Err(UnexpectedEof.into());
    }
    Ok(buf)
}

/// Reads `len` number of bytes from a byte stream and converts them to a string.
pub fn read_str<R: Read>(r: &mut R, len: u64) -> Result<String> {
    let bytes = read_bytes(r, len)?;
    Ok(String::from_utf8(bytes)?)
}

/// Reads a list of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_list<R: Read + Seek, T: Decode<DagCbor>>(r: &mut R, len: u64) -> Result<Vec<T>> {
    let len = usize::try_from(len).map_err(|_| LengthOutOfRange::new::<usize>())?;
    // Limit up-front allocations to 16KiB as the length is user controlled.
    //
    // Can't make this "const" because the generic, but it _should_ be known at compile time.
    let max_alloc = (16 * 1024) / std::mem::size_of::<T>();

    let mut list: Vec<T> = Vec::with_capacity(len.min(max_alloc));
    for _ in 0..len {
        list.push(T::decode(DagCbor, r)?);
    }
    Ok(list)
}

/// Reads a map of any type that implements `TryReadCbor` from a stream of cbor encoded bytes.
pub fn read_map<R: Read + Seek, K: Decode<DagCbor> + Ord, T: Decode<DagCbor>>(
    r: &mut R,
    len: u64,
) -> Result<BTreeMap<K, T>> {
    let len = usize::try_from(len).map_err(|_| LengthOutOfRange::new::<usize>())?;
    let mut map: BTreeMap<K, T> = BTreeMap::new();
    for _ in 0..len {
        let key = K::decode(DagCbor, r)?;
        let value = T::decode(DagCbor, r)?;
        map.insert(key, value);
    }
    Ok(map)
}

/// Reads a cid from a stream of cbor encoded bytes.
pub fn read_link<R: Read + Seek>(r: &mut R) -> Result<Cid> {
    let major = read_major(r)?;
    if major.kind() != MajorKind::ByteString {
        return Err(UnexpectedCode::new::<Cid>(major.0).into());
    }
    let len = read_uint(r, major)?;
    if len < 1 {
        return Err(LengthOutOfRange::new::<Cid>().into());
    }

    let mut r = r.take(len);

    // skip the first byte per
    // https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md#links
    let prefix = read_u8(&mut r)?;
    if prefix != 0 {
        return Err(InvalidCidPrefix(prefix).into());
    }

    // Read the CID. No need to limit the size, the CID will do this for us.
    let cid = Cid::read_bytes(&mut r)?;

    // Make sure we've read the entire CID.
    if r.read(&mut [0u8][..])? != 0 {
        return Err(LengthOutOfRange::new::<Cid>().into());
    }

    Ok(cid)
}

fn read_major<R: Read>(r: &mut R) -> Result<Major> {
    Ok(Major::try_from(read_u8(r)?)?)
}

fn read_uint<R: Read>(r: &mut R, major: Major) -> Result<u64> {
    const MAX_SHORT: u64 = 23;
    const MAX_1BYTE: u64 = u8::MAX as u64;
    const MAX_2BYTE: u64 = u16::MAX as u64;
    const MAX_4BYTE: u64 = u32::MAX as u64;
    match major.info() {
        value @ 0..=23 => Ok(value as u64),
        24 => match read_u8(r)? as u64 {
            0..=MAX_SHORT => Err(NumberNotMinimal.into()),
            value => Ok(value),
        },
        25 => match read_u16(r)? as u64 {
            0..=MAX_1BYTE => Err(NumberNotMinimal.into()),
            value => Ok(value),
        },
        26 => match read_u32(r)? as u64 {
            0..=MAX_2BYTE => Err(NumberNotMinimal.into()),
            value => Ok(value),
        },
        27 => match read_u64(r)? as u64 {
            0..=MAX_4BYTE => Err(NumberNotMinimal.into()),
            value => Ok(value),
        },
        _ => return Err(UnexpectedCode::new::<usize>(major.0).into()),
    }
}

impl Decode<DagCbor> for bool {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        Ok(match read_major(r)? {
            consts::FALSE => false,
            consts::TRUE => true,
            Major(n) => return Err(UnexpectedCode::new::<Self>(n).into()),
        })
    }
}

macro_rules! impl_num {
    (unsigned $($t:ty),*) => {
        $(
            impl Decode<DagCbor> for $t {
                fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
                    let major = read_major(r)?;
                    if major.kind() != MajorKind::UnsignedInt {
                        return Err(UnexpectedCode::new::<Self>(major.0).into());
                    }
                    let value = read_uint(r, major)?;
                    Self::try_from(value).map_err(|_| NumberOutOfRange::new::<Self>().into())
                }
            }
        )*
    };
    (signed $($t:ty),*) => {
        $(
            impl Decode<DagCbor> for $t {
                fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
                    let major = read_major(r)?;
                    let value = read_uint(r, major)?;
                    match major.kind() {
                        MajorKind::UnsignedInt | MajorKind::NegativeInt => (),
                        _ => return Err(UnexpectedCode::new::<Self>(major.0).into()),
                    };

                    let mut value = Self::try_from(value)
                        .map_err(|_| NumberOutOfRange::new::<Self>())?;
                    if major.kind() == MajorKind::NegativeInt {
                        // This is guaranteed to not overflow.
                        value = -1 - value;
                    }
                    Ok(value)
                }
            }
        )*
    };
}

impl_num!(unsigned u8, u16, u32, u64, u128);
impl_num!(signed i8, i16, i32, i64, i128);

impl Decode<DagCbor> for f32 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        // TODO: We don't accept f16
        // TODO: By IPLD spec, we shouldn't accept f32 either...
        let num = match read_major(r)? {
            consts::F32 => f32::from_bits(read_u32(r)?),
            consts::F64 => {
                let num = f64::from_bits(read_u64(r)?);
                let converted = num as Self;
                if f64::from(converted) != num {
                    return Err(NumberOutOfRange::new::<Self>().into());
                }
                converted
            }
            Major(v) => return Err(UnexpectedCode::new::<Self>(v).into()),
        };
        if !num.is_finite() {
            return Err(NumberOutOfRange::new::<Self>().into());
        }
        Ok(num)
    }
}

impl Decode<DagCbor> for f64 {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        // TODO: We don't accept f16
        // TODO: By IPLD spec, we shouldn't accept f32 either...
        let num = match read_major(r)? {
            consts::F32 => f32::from_bits(read_u32(r)?).into(),
            consts::F64 => f64::from_bits(read_u64(r)?),
            Major(v) => return Err(UnexpectedCode::new::<Self>(v).into()),
        };
        // This is by IPLD spec, but is it widely used?
        if !num.is_finite() {
            return Err(NumberOutOfRange::new::<Self>().into());
        }
        Ok(num)
    }
}

impl Decode<DagCbor> for String {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_major(r)?;
        if major.kind() != MajorKind::TextString {
            return Err(UnexpectedCode::new::<Self>(major.0).into());
        }
        let len = read_uint(r, major)?;
        read_str(r, len)
    }
}

impl Decode<DagCbor> for Cid {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_major(r)?;
        if major.kind() == MajorKind::Tag {
            match read_uint(r, major)? {
                42 => read_link(r),
                tag => Err(UnknownTag(tag).into()),
            }
        } else {
            Err(UnexpectedCode::new::<Self>(major.0).into())
        }
    }
}

impl Decode<DagCbor> for Box<[u8]> {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_major(r)?;
        if major.kind() != MajorKind::ByteString {
            return Err(UnexpectedCode::new::<Self>(major.0).into());
        }
        let len = read_uint(r, major)?;
        Ok(read_bytes(r, len)?.into_boxed_slice())
    }
}

impl<T: Decode<DagCbor>> Decode<DagCbor> for Option<T> {
    fn decode<R: Read + Seek>(c: DagCbor, r: &mut R) -> Result<Self> {
        let result = match read_major(r)? {
            consts::NULL => None,
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
        let major = read_major(r)?;
        if major.kind() != MajorKind::Array {
            return Err(UnexpectedCode::new::<Self>(major.0).into());
        }
        let len = read_uint(r, major)?;
        read_list(r, len)
    }
}

impl<K: Decode<DagCbor> + Ord, T: Decode<DagCbor>> Decode<DagCbor> for BTreeMap<K, T> {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_major(r)?;
        if major.kind() != MajorKind::Map {
            return Err(UnexpectedCode::new::<Self>(major.0).into());
        }

        let len = read_uint(r, major)?;
        read_map(r, len)
    }
}

impl Decode<DagCbor> for Ipld {
    fn decode<R: Read + Seek>(_: DagCbor, r: &mut R) -> Result<Self> {
        let major = read_major(r)?;
        let ipld = match major.kind() {
            MajorKind::UnsignedInt => Self::Integer(read_uint(r, major)? as i128),
            MajorKind::NegativeInt => Self::Integer(-1 - read_uint(r, major)? as i128),
            MajorKind::ByteString => {
                let len = read_uint(r, major)?;
                Self::Bytes(read_bytes(r, len)?)
            }
            MajorKind::TextString => {
                let len = read_uint(r, major)?;
                Self::String(read_str(r, len)?)
            }
            MajorKind::Array => {
                let len = read_uint(r, major)?;
                Self::List(read_list(r, len)?)
            }
            MajorKind::Map => {
                let len = read_uint(r, major)?;
                #[cfg(feature = "unleashed")]
                if len > 0 {
                    let pos = r.seek(SeekFrom::Current(0))?;
                    if let Ok(map) = read_map(r, len) {
                        return Ok(Self::IntegerMap(map));
                    }
                    r.seek(SeekFrom::Start(pos))?;
                }
                Self::StringMap(read_map(r, len)?)
            }
            MajorKind::Tag => {
                let value = read_uint(r, major)?;
                if value == 42 {
                    Self::Link(read_link(r)?)
                } else {
                    #[cfg(not(feature = "unleashed"))]
                    return Err(UnknownTag(value).into());
                    #[cfg(feature = "unleashed")]
                    Self::Tag(value as _, Box::new(Self::decode(DagCbor, r)?))
                }
            }
            MajorKind::Other => match major {
                consts::FALSE => Self::Bool(false),
                consts::TRUE => Self::Bool(true),
                consts::NULL => Self::Null,
                consts::F32 => Self::Float(f32::from_bits(read_u32(r)?).into()),
                consts::F64 => Self::Float(f64::from_bits(read_u64(r)?).into()),
                Major(v) => return Err(UnexpectedCode::new::<Self>(v).into()),
            },
        };
        Ok(ipld)
    }
}

impl References<DagCbor> for Ipld {
    fn references<R: Read + Seek, E: Extend<Cid>>(
        _: DagCbor,
        r: &mut R,
        set: &mut E,
    ) -> Result<()> {
        let mut remaining: u64 = 1;
        while remaining > 0 {
            remaining -= 1;
            let major = read_major(r)?;
            match major.kind() {
                MajorKind::UnsignedInt | MajorKind::NegativeInt | MajorKind::Other => {
                    // TODO: validate ints & floats?
                    r.seek(SeekFrom::Current(major.len() as i64))?;
                }
                MajorKind::ByteString | MajorKind::TextString => {
                    // TODO: validate utf8?
                    // We could just reject this case, but we can't just play it fast and loose and
                    // wrap. We might as well just try to seek (and likely fail).
                    let mut offset = read_uint(r, major)?;
                    while offset > i64::MAX as u64 {
                        r.seek(SeekFrom::Current(i64::MAX))?;
                        offset -= i64::MAX as u64;
                    }
                    r.seek(SeekFrom::Current(offset as i64))?;
                }
                MajorKind::Array => {
                    remaining = remaining
                        .checked_add(read_uint(r, major)?)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                }
                MajorKind::Map => {
                    // TODO: consider using a checked "monad" type to simplify.
                    let items = read_uint(r, major)?
                        .checked_mul(2)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                    remaining = remaining
                        .checked_add(items)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                }
                MajorKind::Tag => match read_uint(r, major)? {
                    42 => set.extend(std::iter::once(read_link(r)?)),
                    _ => {
                        remaining = remaining
                            .checked_add(1)
                            .ok_or_else(LengthOutOfRange::new::<Self>)?;
                    }
                },
            };
        }
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
        match major {
            0x80 => {}
            _ => {
                return Err(UnexpectedCode::new::<Self>(major).into());
            }
        };
        Ok(())
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
        let mut remaining: u64 = 1;
        while remaining > 0 {
            remaining -= 1;
            let major = read_major(r)?;
            match major.kind() {
                MajorKind::UnsignedInt | MajorKind::NegativeInt | MajorKind::Other => {
                    // TODO: validate?
                    // minimal integer, valid float, etc?
                    r.seek(SeekFrom::Current(major.len() as i64))?;
                }
                MajorKind::ByteString | MajorKind::TextString => {
                    // We could just reject this case, but we can't just play it fast and loose and
                    // wrap. We might as well just try to seek (and likely fail).
                    let mut offset = read_uint(r, major)?;
                    while offset > i64::MAX as u64 {
                        r.seek(SeekFrom::Current(i64::MAX))?;
                        offset -= i64::MAX as u64;
                    }
                    // TODO: validate utf8?
                    r.seek(SeekFrom::Current(offset as i64))?;
                }
                MajorKind::Array => {
                    remaining = remaining
                        .checked_add(read_uint(r, major)?)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                }
                MajorKind::Map => {
                    // TODO: consider using a checked "monad" type to simplify.
                    let items = read_uint(r, major)?
                        .checked_mul(2)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                    remaining = remaining
                        .checked_add(items)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                }
                MajorKind::Tag => {
                    // TODO: validate tag?
                    r.seek(SeekFrom::Current(major.len() as i64))?;
                    remaining = remaining
                        .checked_add(1)
                        .ok_or_else(LengthOutOfRange::new::<Self>)?;
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::UnexpectedEof, DagCborCodec};
    use libipld_core::codec::Codec;

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
        DagCborCodec
            .decode::<Ipld>(&bytes)
            .expect_err("should have failed to decode indefinit length map");
    }

    #[test]
    fn bad_list() {
        let bytes = [
            0x5b, // Byte string with an 8 byte length
            0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // very long
            0x01, // but only one byte.
        ];
        DagCborCodec
            .decode::<Ipld>(&bytes)
            .expect_err("decoding large truncated buffer should have failed")
            .downcast::<UnexpectedEof>()
            .expect("expected an unexpected eof");
    }

    #[test]
    fn tuples() -> Result<()> {
        let data = ();
        let bytes = DagCborCodec.encode(&data)?;
        let _data2: () = DagCborCodec.decode(&bytes)?;

        let data = ("hello".to_string(),);
        let bytes = DagCborCodec.encode(&data)?;
        let data2: (String,) = DagCborCodec.decode(&bytes)?;
        assert_eq!(data, data2);

        let data = ("hello".to_string(), "world".to_string());
        let bytes = DagCborCodec.encode(&data)?;
        let data2: (String, String) = DagCborCodec.decode(&bytes)?;
        assert_eq!(data, data2);

        let data = ("hello".to_string(), "world".to_string(), 42);
        let bytes = DagCborCodec.encode(&data)?;
        let data2: (String, String, u32) = DagCborCodec.decode(&bytes)?;
        assert_eq!(data, data2);

        let data = ("hello".to_string(), "world".to_string(), 42, 64);
        let bytes = DagCborCodec.encode(&data)?;
        let data2: (String, String, u32, u8) = DagCborCodec.decode(&bytes)?;
        assert_eq!(data, data2);

        Ok(())
    }
}
