//! CBOR encoder.
use crate::{CborResult as Result};
pub use async_std::io::Write;
use async_std::prelude::*;
use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use libipld_base::cid::Cid;
use libipld_base::ipld::Ipld;
use std::collections::BTreeMap;

#[async_trait]
pub trait WriteCbor {
    /// `Write`s the `major` (unless it's part of the value) and remaining
    /// header, returning the number of bytes written.
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8>;

    /// `Write`s the actual bytes for the type, returning the number of bytes
    /// written.
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize>;

    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
        self.write_prefix(w).await?;
        self.write_type(w).await?;
        Ok(())
    }
}

#[async_trait]
impl WriteCbor for () {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, _w: &mut W) -> Result<u8> {
        Ok(0)
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        w.write_all(&[0xf6]).await?;
        Ok(1)
    }
}

#[async_trait]
impl WriteCbor for bool {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, _w: &mut W) -> Result<u8> {
        Ok(0)
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        let buf = if *self { [0xf5] } else { [0xf4] };
        w.write_all(&buf).await?;
        Ok(1)
    }
}

#[async_trait]
impl WriteCbor for u8 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u8_header(w, 0, *self).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u8(w, 0, *self).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for u16 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u16_header(w, 0, *self).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u16(w, 0, *self).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for u32 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u32_header(w, 0, *self).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u32(w, 0, *self).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for u64 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u64_header(w, 0, *self).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u64(w, 0, *self).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for i8 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u8_header(w, 1, -(*self + 1) as u8).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u8(w, 1, -(*self + 1) as u8).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for i16 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u16_header(w, 1, -(*self + 1) as u16).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u16(w, 1, -(*self + 1) as u16).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for i32 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u32_header(w, 1, -(*self + 1) as u32).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u32(w, 1, -(*self + 1) as u32).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for i64 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u64_header(w, 1, -(*self + 1) as u64).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        Ok(write_u64(w, 1, -(*self + 1) as u64).await? as usize)
    }
}

#[async_trait]
impl WriteCbor for f32 {
    #[inline]
    #[allow(clippy::float_cmp)]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        if self.is_infinite() || self.is_nan() {
            w.write_all(&[0xf9]).await?;
        } else {
            w.write_all(&[0xfa]).await?;
        }
        Ok(1)
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        if self.is_infinite() {
            if self.is_sign_positive() {
                w.write_all(&[0x7c, 0x00]).await?;
            } else {
                w.write_all(&[0xfc, 0x00]).await?;
            }
            Ok(2)
        } else if self.is_nan() {
            w.write_all(&[0x7e, 0x00]).await?;
            Ok(2)
        } else {
            let mut buf = [0, 0, 0, 0];
            BigEndian::write_f32(&mut buf, *self);
            w.write_all(&buf).await?;
            Ok(4)
        }
    }
}

#[async_trait]
impl WriteCbor for f64 {
    #[inline]
    #[allow(clippy::float_cmp)]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        if !self.is_finite() || f64::from(*self as f32) == *self {
            (*self as f32).write_prefix(w).await
        } else {
            w.write_all(&[0xfb]).await?;
            Ok(1)
        }
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        if !self.is_finite() || f64::from(*self as f32) == *self {
            (*self as f32).write_type(w).await
        } else {
            let mut buf = [0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_f64(&mut buf, *self);
            w.write_all(&buf).await?;
            Ok(8)
        }
    }
}

#[async_trait]
impl WriteCbor for [u8] {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u64(w, 2, self.len() as u64).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        w.write_all(self).await?;
        Ok(self.len() as usize)
    }
}

#[async_trait]
impl WriteCbor for Box<[u8]> {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        self.as_ref().write_prefix(w).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        self.as_ref().write_type(w).await
    }
}

#[cfg(feature = "bytes_")]
#[async_trait]
impl WriteCbor for bytes::Bytes {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        self.as_ref().write_prefix(w).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        self.as_ref().write_type(w).await
    }
}

#[async_trait]
impl WriteCbor for str {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_u64(w, 3, self.len() as u64).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        w.write_all(self.as_bytes()).await?;
        Ok(self.len() as usize)
    }
}

#[async_trait]
impl WriteCbor for String {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        self.as_str().write_prefix(w).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        self.as_str().write_type(w).await
    }
}

#[async_trait]
impl WriteCbor for i128 {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        unimplemented!()
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        unimplemented!()
    }
}

#[async_trait]
impl WriteCbor for Cid {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_tag(w, 42).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        let bytes = self.to_bytes();
        let len = bytes.as_slice().write_prefix(w).await?;
        Ok(len as usize + bytes.as_slice().write_type(w).await?)
    }
}

#[async_trait]
impl<T: WriteCbor + Send + Sync> WriteCbor for Option<T> {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        match self {
            Some(value) => value.write_prefix(w).await,
            None => <()>::write_prefix(&(), w).await,
        }
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        match self {
            Some(value) => value.write_type(w).await,
            None => <()>::write_type(&(), w).await,
        }
    }
}

#[async_trait]
impl<T: WriteCbor + Send + Sync> WriteCbor for Vec<T> {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_list_len(w, self.len()).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        let mut sum: usize = 0;
        for value in self {
            sum += value.write_prefix(w).await? as usize;
            sum += value.write_type(w).await?;
        }
        Ok(sum)
    }
}

#[async_trait]
impl<T: 'static + WriteCbor + Send + Sync> WriteCbor for BTreeMap<String, T> {
    #[inline]
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        write_map_len(w, self.len()).await
    }

    #[inline]
    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        let mut sum: usize = 0;
        for (key, value) in self {
            sum += key.write_prefix(w).await? as usize;
            sum += key.write_type(w).await?;
            sum += value.write_prefix(w).await? as usize;
            sum += value.write_type(w).await?;
        }
        Ok(sum)
    }
}

// TODO: standard iterators
// #[async_trait]
// impl<T: WriteCbor + Send + Sync, E: WriteCbor> WriteCbor for T where T: Iterator<Item = E> {
//     #[inline]
//     default async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
//         write_u64(w, 4, self.len() as u64).await?;
//         for value in self {
//             value.write_cbor(w).await?;
//         }
//         Ok(())
//     }
// }

// TODO: map iterators
// #[async_trait]
// impl<T: WriteCbor + Send + Sync, V: WriteCbor> WriteCbor for T where T: Iterator<Item = (String, V)> {
//     #[inline]
//     default async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
//         write_u64(w, 4, self.len() as u64).await?;
//         for value in self {
//             value.write_cbor(w).await?;
//         }
//         Ok(())
//     }
// }

// TODO: list streams
// #[async_trait]
// impl<T: WriteCbor + Send + Sync, V: WriteCbor> WriteCbor for T where T: Iterator<Item = (String, V)> {
//     #[inline]
//     default async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
//         write_u64(w, 4, self.len() as u64).await?;
//         for value in self {
//             value.write_cbor(w).await?;
//         }
//         Ok(())
//     }
// }

//#[async_trait]
//impl<T: WriteCbor + Send + Sync> WriteCbor for Box<T> {
//    #[inline]
//    fn offsets(&self) -> Result<(u8, usize)> {
//        T::offsets(self.as_ref())
//    }
//
//    #[inline]
//    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<()> {
//        T::write_cbor(self.as_ref(), w).await
//    }
//}

#[async_trait]
impl WriteCbor for Ipld {
    async fn write_prefix<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<u8> {
        match self {
            Ipld::Null => <()>::write_prefix(&(), w).await,
            Ipld::Bool(b) => b.write_prefix(w).await,
            Ipld::Integer(i) => i.write_prefix(w).await,
            Ipld::Float(f) => f.write_prefix(w).await,
            Ipld::Bytes(b) => <[u8]>::write_prefix(b, w).await,
            Ipld::String(s) => s.write_prefix(w).await,
            Ipld::List(l) => l.write_prefix(w).await,
            Ipld::Map(m) => m.write_prefix(w).await,
            Ipld::Link(c) => c.write_prefix(w).await,
        }
    }

    async fn write_type<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<usize> {
        match self {
            Ipld::Null => <()>::write_type(&(), w).await,
            Ipld::Bool(b) => b.write_type(w).await,
            Ipld::Integer(i) => i.write_type(w).await,
            Ipld::Float(f) => f.write_type(w).await,
            Ipld::Bytes(b) => b.as_slice().write_type(w).await,
            Ipld::String(s) => s.write_type(w).await,
            Ipld::List(l) => l.write_type(w).await,
            Ipld::Map(m) => m.write_type(w).await,
            Ipld::Link(c) => c.write_type(w).await,
        }
    }
}

// helpers

#[inline]
async fn write_u8_header<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u8) -> Result<u8> {
    if value <= 0x17 {
        Ok(0)
    } else {
        w.write_all(&[major << 5 | 24]).await?;
        Ok(1)
    }
}

#[inline]
async fn write_u8<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u8) -> Result<u8> {
    if value <= 0x17 {
        w.write_all(&[major << 5 | value]).await?;
        Ok(1)
    } else {
        w.write_all(&[value]).await?;
        Ok(1)
    }
}

#[inline]
async fn write_u16_header<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u16) -> Result<u8> {
    if value <= u16::from(u8::max_value()) {
        write_u8_header(w, major, value as u8).await
    } else {
        w.write_all(&[major << 5 | 25]).await?;
        Ok(1)
    }
}

#[inline]
async fn write_u16<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u16) -> Result<u8> {
    if value <= u16::from(u8::max_value()) {
        write_u8(w, major, value as u8).await
    } else {
        let mut buf = [0, 0];
        BigEndian::write_u16(&mut buf, value);
        w.write_all(&buf).await?;
        Ok(2)
    }
}

#[inline]
async fn write_u32_header<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u32) -> Result<u8> {
    if value <= u32::from(u16::max_value()) {
        write_u16_header(w, major, value as u16).await
    } else {
        w.write_all(&[major << 5 | 26]).await?;
        Ok(1)
    }
}

#[inline]
async fn write_u32<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u32) -> Result<u8> {
    if value <= u32::from(u16::max_value()) {
        write_u16(w, major, value as u16).await
    } else {
        let mut buf = [0, 0, 0, 0];
        BigEndian::write_u32(&mut buf, value);
        w.write_all(&buf).await?;
        Ok(4)
    }
}

#[inline]
async fn write_u64_header<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u64) -> Result<u8> {
    if value <= u64::from(u32::max_value()) {
        write_u32_header(w, major, value as u32).await
    } else {
        w.write_all(&[major << 5 | 27]).await?;
        Ok(1)
    }
}

#[inline]
async fn write_u64<W: Write + Unpin + Send>(w: &mut W, major: u8, value: u64) -> Result<u8> {
    if value <= u64::from(u32::max_value()) {
        write_u32(w, major, value as u32).await
    } else {
        let mut buf = [0, 0, 0, 0, 0, 0, 0, 0];
        BigEndian::write_u64(&mut buf, value);
        w.write_all(&buf).await?;
        Ok(8)
    }
}

#[inline]
pub async fn write_list_len<W: Write + Unpin + Send>(w: &mut W, len: usize) -> Result<u8> {
    write_u64(w, 4, len as u64).await
}

#[inline]
pub async fn write_map_len<W: Write + Unpin + Send>(w: &mut W, len: usize) -> Result<u8> {
    write_u64(w, 5, len as u64).await
}

#[inline]
pub async fn write_tag<W: Write + Unpin + Send>(w: &mut W, tag: u64) -> Result<u8> {
    write_u64(w, 6, tag).await
}
