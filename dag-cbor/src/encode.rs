//! CBOR encoder.
use crate::{CborError, CborResult as Result};
use byteorder::{BigEndian, ByteOrder};
use libipld_base::cid::Cid;
use libipld_base::ipld::Ipld;
use std::collections::BTreeMap;
pub use std::io::Write;

#[inline]
pub fn write_null<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(&[0xf6])?;
    Ok(())
}

#[inline]
pub fn write_u8<W: Write>(w: &mut W, major: u8, value: u8) -> Result<()> {
    if value <= 0x17 {
        let buf = [major << 5 | value];
        w.write_all(&buf)?;
    } else {
        let buf = [major << 5 | 24, value];
        w.write_all(&buf)?;
    }
    Ok(())
}

#[inline]
pub fn write_u16<W: Write>(w: &mut W, major: u8, value: u16) -> Result<()> {
    if value <= u16::from(u8::max_value()) {
        write_u8(w, major, value as u8)?;
    } else {
        let mut buf = [major << 5 | 25, 0, 0];
        BigEndian::write_u16(&mut buf[1..], value);
        w.write_all(&buf)?;
    }
    Ok(())
}

#[inline]
pub fn write_u32<W: Write>(w: &mut W, major: u8, value: u32) -> Result<()> {
    if value <= u32::from(u16::max_value()) {
        write_u16(w, major, value as u16)?;
    } else {
        let mut buf = [major << 5 | 26, 0, 0, 0, 0];
        BigEndian::write_u32(&mut buf[1..], value);
        w.write_all(&buf)?;
    }
    Ok(())
}

#[inline]
pub fn write_u64<W: Write>(w: &mut W, major: u8, value: u64) -> Result<()> {
    if value <= u64::from(u32::max_value()) {
        write_u32(w, major, value as u32)?;
    } else {
        let mut buf = [major << 5 | 27, 0, 0, 0, 0, 0, 0, 0, 0];
        BigEndian::write_u64(&mut buf[1..], value);
        w.write_all(&buf)?;
    }
    Ok(())
}

#[inline]
pub fn write_tag<W: Write>(w: &mut W, tag: u64) -> Result<()> {
    write_u64(w, 6, tag)
}

pub trait WriteCbor {
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()>;
}

impl WriteCbor for bool {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        let buf = if *self { [0xf5] } else { [0xf4] };
        w.write_all(&buf)?;
        Ok(())
    }
}

impl WriteCbor for u8 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u8(w, 0, *self)
    }
}

impl WriteCbor for u16 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u16(w, 0, *self)
    }
}

impl WriteCbor for u32 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u32(w, 0, *self)
    }
}

impl WriteCbor for u64 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 0, *self)
    }
}

impl WriteCbor for i8 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u8(w, 1, -(*self + 1) as u8)
    }
}

impl WriteCbor for i16 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u16(w, 1, -(*self + 1) as u16)
    }
}

impl WriteCbor for i32 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u32(w, 1, -(*self + 1) as u32)
    }
}

impl WriteCbor for i64 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 1, -(*self + 1) as u64)
    }
}

impl WriteCbor for f32 {
    #[inline]
    #[allow(clippy::float_cmp)]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        if self.is_infinite() {
            if self.is_sign_positive() {
                w.write_all(&[0xf9, 0x7c, 0x00])?;
            } else {
                w.write_all(&[0xf9, 0xfc, 0x00])?;
            }
        } else if self.is_nan() {
            w.write_all(&[0xf9, 0x7e, 0x00])?;
        } else {
            let mut buf = [0xfa, 0, 0, 0, 0];
            BigEndian::write_f32(&mut buf[1..], *self);
            w.write_all(&buf)?;
        }
        Ok(())
    }
}

impl WriteCbor for f64 {
    #[inline]
    #[allow(clippy::float_cmp)]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        if !self.is_finite() || f64::from(*self as f32) == *self {
            let value = *self as f32;
            value.write_cbor(w)?;
        } else {
            let mut buf = [0xfb, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_f64(&mut buf[1..], *self);
            w.write_all(&buf)?;
        }
        Ok(())
    }
}

impl WriteCbor for [u8] {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 2, self.len() as u64)?;
        w.write_all(self)?;
        Ok(())
    }
}

impl WriteCbor for str {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 3, self.len() as u64)?;
        w.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl WriteCbor for String {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        self.as_str().write_cbor(w)
    }
}

impl WriteCbor for i128 {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        if *self < 0 {
            if -(*self + 1) > u64::max_value() as i128 {
                return Err(CborError::NumberOutOfRange);
            }
            write_u64(w, 1, -(*self + 1) as u64)?;
        } else {
            if *self > u64::max_value() as i128 {
                return Err(CborError::NumberOutOfRange);
            }
            write_u64(w, 0, *self as u64)?;
        }
        Ok(())
    }
}

impl WriteCbor for Cid {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_tag(w, 42)?;
        // insert zero byte per https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md#links
        let bytes = self.to_bytes();
        write_u64(w, 2, (bytes.len() + 1) as u64)?;
        w.write_all(&[0])?;
        w.write_all(&bytes)?;
        Ok(())
    }
}

impl<T: WriteCbor> WriteCbor for Option<T> {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        if let Some(value) = self {
            value.write_cbor(w)?;
        } else {
            write_null(w)?;
        }
        Ok(())
    }
}

impl<T: WriteCbor> WriteCbor for Vec<T> {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 4, self.len() as u64)?;
        for value in self {
            value.write_cbor(w)?;
        }
        Ok(())
    }
}

impl<T: WriteCbor + 'static> WriteCbor for BTreeMap<String, T> {
    #[inline]
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 5, self.len() as u64)?;
        for (k, v) in self {
            k.write_cbor(w)?;
            v.write_cbor(w)?;
        }
        Ok(())
    }
}

impl WriteCbor for Ipld {
    fn write_cbor<W: Write>(&self, w: &mut W) -> Result<()> {
        match self {
            Ipld::Null => write_null(w),
            Ipld::Bool(b) => b.write_cbor(w),
            Ipld::Integer(i) => i.write_cbor(w),
            Ipld::Float(f) => f.write_cbor(w),
            Ipld::Bytes(b) => b.as_slice().write_cbor(w),
            Ipld::String(s) => s.as_str().write_cbor(w),
            Ipld::List(l) => l.write_cbor(w),
            Ipld::Map(m) => m.write_cbor(w),
            Ipld::Link(c) => c.write_cbor(w),
        }
    }
}
