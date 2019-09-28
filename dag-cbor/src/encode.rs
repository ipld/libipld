//! CBOR encoder.
use crate::{CborError, CborResult as Result};
use async_std::io::Write as _;
use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
pub use futures_io::AsyncWrite as Write;
use libipld_base::cid::Cid;
use libipld_base::ipld::Ipld;
use std::collections::BTreeMap;

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

#[async_trait]
pub trait WriteCbor {
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()>;
}

#[async_trait]
impl WriteCbor for bool {
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
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u8(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for u16 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u16(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for u32 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u32(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for u64 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 0, *self).await
    }
}

#[async_trait]
impl WriteCbor for i8 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u8(w, 1, -(*self + 1) as u8).await
    }
}

#[async_trait]
impl WriteCbor for i16 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u16(w, 1, -(*self + 1) as u16).await
    }
}

#[async_trait]
impl WriteCbor for i32 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u32(w, 1, -(*self + 1) as u32).await
    }
}

#[async_trait]
impl WriteCbor for i64 {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 1, -(*self + 1) as u64).await
    }
}

#[async_trait]
impl WriteCbor for f32 {
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
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 2, self.len() as u64).await?;
        w.write_all(self).await?;
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for str {
    #[inline]
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 3, self.len() as u64).await?;
        w.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for i128 {
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
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        write_u64(w, 4, self.len() as u64).await?;
        for value in self {
            value.write_cbor(w).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T: WriteCbor + Send + Sync + 'static> WriteCbor for BTreeMap<String, T> {
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
