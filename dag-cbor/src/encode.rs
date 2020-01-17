//! CBOR encoder.
use crate::{CborError, CborResult as Result};
pub use async_std::io::Write;
use async_std::prelude::*;
use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use libipld_base::cid::Cid;
use libipld_base::ipld::Ipld;
use std::collections::BTreeMap;

#[async_trait]
pub trait WriteCbor {
    /// Returns the number of bytes written for the header (major +
    /// length), and the number of bytes the type actually occupies.
    fn offsets(&self) -> Result<(u8, usize)>;

    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()>;
}

#[async_trait]
impl WriteCbor for () {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        Ok((0, 1))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_null(w).await
    }
}

#[async_trait]
impl WriteCbor for bool {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        Ok((0, 1))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        let buf = if *self { [0xf5] } else { [0xf4] };
        w.write_all(&buf).await?;
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for u8 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u8_byte_offset(*self);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u8(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for u16 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u16_byte_offset(*self);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u16(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for u32 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u32_byte_offset(*self);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u32(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for u64 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u64_byte_offset(*self);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for i8 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u8_byte_offset(-(*self + 1) as u8);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u8(w, 1, -(*self + 1) as u8).await
    }
}

#[async_trait]
impl WriteCbor for i16 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u16_byte_offset(-(*self + 1) as u16);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u16(w, 1, -(*self + 1) as u16).await
    }
}

#[async_trait]
impl WriteCbor for i32 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u32_byte_offset(-(*self + 1) as u32);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u32(w, 1, -(*self + 1) as u32).await
    }
}

#[async_trait]
impl WriteCbor for i64 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (offset, len) = u64_byte_offset(-(*self + 1) as u64);
        Ok((offset, len as usize))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 1, -(*self + 1) as u64).await
    }
}

#[async_trait]
impl WriteCbor for f32 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        if self.is_infinite() || self.is_nan() {
            Ok((1, 2))
        } else {
            Ok((1, 4))
        }
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        if self.is_infinite() {
            if self.is_sign_positive() {
                w.write_all(&[0xf9, 0x7c, 0x00]).await?;
            } else {
                w.write_all(&[0xf9, 0xfc, 0x00]).await?;
            }
        } else if self.is_nan() {
            w.write_all(&[0xf9, 0x7e, 0x00]).await?;
        } else {
            let mut buf = [0xfa, 0, 0, 0, 0];
            BigEndian::write_f32(&mut buf[1..], *self);
            w.write_all(&buf).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for f64 {
    #[inline]
    #[allow(clippy::float_cmp)]
    fn offsets(&self) -> Result<(u8, usize)> {
        if !self.is_finite() || f64::from(*self as f32) == *self {
            (*self as f32).offsets()
        } else {
            Ok((1, 8))
        }
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        if !self.is_finite() || f64::from(*self as f32) == *self {
            let value = *self as f32;
            value.write_cbor(w).await?;
        } else {
            let mut buf = [0xfb, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_f64(&mut buf[1..], *self);
            w.write_all(&buf).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for [u8] {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (tag, len) = u64_byte_offset(self.len() as u64);
        Ok((tag + len, self.len()))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 2, self.len() as u64).await?;
        w.write_all(self).await?;
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for Box<[u8]> {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        self.as_ref().offsets()
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        self.as_ref().write_cbor(w).await
    }
}

#[cfg(feature = "bytes_")]
#[async_trait]
impl WriteCbor for bytes::Bytes {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        self.as_ref().offsets()
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        self.as_ref().write_cbor(w).await
    }
}

#[async_trait]
impl WriteCbor for str {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (tag, len) = u64_byte_offset(self.len() as u64);
        Ok((tag + len, self.len()))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 3, self.len() as u64).await?;
        w.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for String {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        self.as_str().offsets()
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        self.as_str().write_cbor(w).await
    }
}

#[async_trait]
impl WriteCbor for i128 {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        if *self < 0 {
            if -(*self + 1) > u64::max_value() as i128 {
                Err(CborError::NumberOutOfRange)
            } else {
                let (offset, len) = u64_byte_offset(-(*self + 1) as u64);
                Ok((offset, len as usize))
            }
        } else if *self > u64::max_value() as i128 {
            Err(CborError::NumberOutOfRange)
        } else {
            let (offset, len) = u64_byte_offset(*self as u64);
            Ok((offset, len as usize))
        }
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        if *self < 0 {
            if -(*self + 1) > u64::max_value() as i128 {
                return Err(CborError::NumberOutOfRange);
            }
            write_u64(w, 1, -(*self + 1) as u64).await?;
        } else {
            if *self > u64::max_value() as i128 {
                return Err(CborError::NumberOutOfRange);
            }
            write_u64(w, 0, *self as u64).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for Cid {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let digest = self.hash().digest();
        let (byte_offset, bytes_len) = digest.offsets()?;
        let header_len = {
            let (tag, len) = tag_offset(42);
            tag + len + byte_offset
        };
        Ok((header_len, bytes_len))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_tag(w, 42).await?;
        let bytes = self.to_bytes();
        bytes.as_slice().write_cbor(w).await?;
        Ok(())
    }
}

#[async_trait]
impl<T: WriteCbor + Send + Sync> WriteCbor for Option<T> {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        if let Some(value) = self {
            value.offsets()
        } else {
            <()>::offsets(&())
        }
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        if let Some(value) = self {
            value.write_cbor(w).await?;
        } else {
            write_null(w).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T: WriteCbor + Send + Sync> WriteCbor for Vec<T> {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (tag, len) = u64_byte_offset(self.len() as u64);
        Ok((
            tag + len,
            self.iter().try_fold(0, |sum, t| {
                let (offset, len) = t.offsets()?;
                Ok(sum + len + offset as usize) as Result<usize>
            })?,
        ))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 4, self.len() as u64).await?;
        for value in self {
            value.write_cbor(w).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T: 'static + WriteCbor + Send + Sync> WriteCbor for BTreeMap<String, T> {
    #[inline]
    fn offsets(&self) -> Result<(u8, usize)> {
        let (tag, len) = u64_byte_offset(self.len() as u64);
        Ok((
            tag + len,
            self.iter().try_fold(0, |sum, (k, v)| {
                let (k_offset, k_len) = k.offsets()?;
                let (v_offset, v_len) = v.offsets()?;
                Ok(sum + k_len + v_len + (k_offset + v_offset) as usize) as Result<usize>
            })?,
        ))
    }

    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 5, self.len() as u64).await?;
        for (k, v) in self {
            k.write_cbor(w).await?;
            v.write_cbor(w).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for Ipld {
    fn offsets(&self) -> Result<(u8, usize)> {
        match self {
            Ipld::Null => <()>::offsets(&()),
            Ipld::Bool(b) => b.offsets(),
            Ipld::Integer(i) => i.offsets(),
            Ipld::Float(f) => f.offsets(),
            Ipld::Bytes(b) => <[u8]>::offsets(b),
            Ipld::String(s) => s.offsets(),
            Ipld::List(l) => l.offsets(),
            Ipld::Map(m) => m.offsets(),
            Ipld::Link(c) => c.offsets(),
        }
    }

    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        match self {
            Ipld::Null => write_null(w).await,
            Ipld::Bool(b) => b.write_cbor(w).await,
            Ipld::Integer(i) => i.write_cbor(w).await,
            Ipld::Float(f) => f.write_cbor(w).await,
            Ipld::Bytes(b) => b.as_slice().write_cbor(w).await,
            Ipld::String(s) => s.as_str().write_cbor(w).await,
            Ipld::List(l) => l.write_cbor(w).await,
            Ipld::Map(m) => m.write_cbor(w).await,
            Ipld::Link(c) => c.write_cbor(w).await,
        }
    }
}

#[inline]
pub async fn write_null<W: Write + Unpin + Send>(w: &mut W) -> Result<()> {
    w.write_all(&[0xf6]).await?;
    Ok(())
}

#[inline]
pub async fn write_u8<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u8) -> Result<()> {
    if value <= 0x17 {
        let buf = [major << 5 | value];
        w.write_all(&buf).await?;
    } else {
        let buf = [major << 5 | 24, value];
        w.write_all(&buf).await?;
    }
    Ok(())
}

#[inline]
pub async fn write_u16<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u16) -> Result<()> {
    if value <= u16::from(u8::max_value()) {
        write_u8(w, major, value as u8).await?;
    } else {
        let mut buf = [major << 5 | 25, 0, 0];
        BigEndian::write_u16(&mut buf[1..], value);
        w.write_all(&buf).await?;
    }
    Ok(())
}

#[inline]
pub async fn write_u32<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u32) -> Result<()> {
    if value <= u32::from(u16::max_value()) {
        write_u16(w, major, value as u16).await?;
    } else {
        let mut buf = [major << 5 | 26, 0, 0, 0, 0];
        BigEndian::write_u32(&mut buf[1..], value);
        w.write_all(&buf).await?;
    }
    Ok(())
}

#[inline]
pub async fn write_u64<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u64) -> Result<()> {
    if value <= u64::from(u32::max_value()) {
        write_u32(w, major, value as u32).await?;
    } else {
        let mut buf = [major << 5 | 27, 0, 0, 0, 0, 0, 0, 0, 0];
        BigEndian::write_u64(&mut buf[1..], value);
        w.write_all(&buf).await?;
    }
    Ok(())
}

#[inline]
pub async fn write_tag<W: Write + Unpin + Send>(w: &mut W, tag: u64) -> Result<()> {
    write_u64(w, 6, tag).await?;
    Ok(())
}

// Byte offset calculators, including `major`.

#[inline]
fn u8_byte_offset(value: u8) -> (u8, u8) {
    if value <= 0x17 {
        (0, 1)
    } else {
        (1, 1)
    }
}

#[inline]
fn u16_byte_offset(value: u16) -> (u8, u8) {
    if value <= u16::from(u8::max_value()) {
        u8_byte_offset(value as u8)
    } else {
        (1, 2)
    }
}

#[inline]
fn u32_byte_offset(value: u32) -> (u8, u8) {
    if value <= u32::from(u16::max_value()) {
        u16_byte_offset(value as u16)
    } else {
        (1, 4)
    }
}

#[inline]
fn u64_byte_offset(value: u64) -> (u8, u8) {
    if value <= u64::from(u32::max_value()) {
        u32_byte_offset(value as u32)
    } else {
        (1, 8)
    }
}

#[inline]
fn tag_offset(tag: u64) -> (u8, u8) {
    u64_byte_offset(tag)
}
