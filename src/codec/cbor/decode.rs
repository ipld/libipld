//! CBOR decoder
use crate::error::Result;
use crate::ipld::Ipld;
use byteorder::{BigEndian, ByteOrder};
use cid::Cid;
use core::convert::TryFrom;
use failure::Fail;
use std::collections::BTreeMap;
use std::io::Read;

#[derive(Debug, Fail)]
pub enum CborError {
    #[fail(display = "Length out of range.")]
    LengthOutOfRange,
    #[fail(display = "Unexpected code.")]
    UnexpectedCode,
    #[fail(display = "Unkown tag.")]
    UnknownTag,
    #[fail(display = "{}", _0)]
    Io(std::io::Error),
}

impl From<std::io::Error> for CborError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

pub struct Decoder<R> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn decode(&mut self) -> Result<Ipld> {
        self.parse_ipld()
    }

    fn read(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(len);
        let reader_ref = self.reader.by_ref();
        let mut taken = reader_ref.take(len as u64);
        taken.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn parse_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(BigEndian::read_u16(&buf))
    }

    fn parse_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(BigEndian::read_u32(&buf))
    }

    fn parse_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(BigEndian::read_u64(&buf))
    }

    fn parse_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        self.read(len)
    }

    fn parse_str(&mut self, len: usize) -> Result<String> {
        let bytes = self.read(len)?;
        let string = std::str::from_utf8(&bytes)?;
        Ok(string.to_string())
    }

    fn parse_list(&mut self, len: usize) -> Result<Vec<Ipld>> {
        let mut list: Vec<Ipld> = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(self.parse_ipld()?);
        }
        Ok(list)
    }

    fn parse_link(&mut self) -> Result<Cid> {
        let tag = self.parse_u8()?;
        if tag != 42 {
            return Err(CborError::UnknownTag.into());
        }
        let ty = self.parse_u8()?;
        if ty != 0x58 {
            return Err(CborError::UnknownTag.into());
        }
        let len = self.parse_u8()?;
        let bytes = self.parse_bytes(len as usize)?;
        Ok(Cid::try_from(bytes)?)
    }

    fn parse_map(&mut self, len: usize) -> Result<BTreeMap<String, Ipld>> {
        let mut map: BTreeMap<String, Ipld> = BTreeMap::new();
        for _ in 0..len {
            let key = self.parse_key()?;
            let value = self.parse_ipld()?;
            map.insert(key, value);
        }
        Ok(map)
    }

    fn parse_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(BigEndian::read_f32(&buf))
    }

    fn parse_f64(&mut self) -> Result<f64> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(BigEndian::read_f64(&buf))
    }

    fn parse_key(&mut self) -> Result<String> {
        let byte = self.parse_u8()?;
        let string = match byte {
            // Major type 3: a text string
            0x60..=0x77 => {
                let len = byte - 0x60;
                self.parse_str(len as usize)?
            }
            0x78 => {
                let len = self.parse_u8()?;
                self.parse_str(len as usize)?
            }
            0x79 => {
                let len = self.parse_u16()?;
                self.parse_str(len as usize)?
            }
            0x7a => {
                let len = self.parse_u32()?;
                self.parse_str(len as usize)?
            }
            0x7b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                self.parse_str(len as usize)?
            }
            _ => return Err(CborError::UnexpectedCode.into()),
        };
        Ok(string)
    }

    fn parse_ipld(&mut self) -> Result<Ipld> {
        let byte = self.parse_u8()?;
        let ipld = match byte {
            // Major type 0: an unsigned integer
            0x00..=0x17 => Ipld::Integer(byte as i128),
            0x18 => Ipld::Integer(self.parse_u8()? as i128),
            0x19 => Ipld::Integer(self.parse_u16()? as i128),
            0x1a => Ipld::Integer(self.parse_u32()? as i128),
            0x1b => Ipld::Integer(self.parse_u64()? as i128),

            // Major type 1: a negative integer
            0x20..=0x37 => {
                let value = byte - 0x20;
                Ipld::Integer(-1 - value as i128)
            }
            0x38 => {
                let value = self.parse_u8()?;
                Ipld::Integer(-1 - value as i128)
            }
            0x39 => {
                let value = self.parse_u16()?;
                Ipld::Integer(-1 - value as i128)
            }
            0x3a => {
                let value = self.parse_u32()?;
                Ipld::Integer(-1 - value as i128)
            }
            0x3b => {
                let value = self.parse_u64()?;
                Ipld::Integer(-1 - value as i128)
            }

            // Major type 2: a byte string
            0x40..=0x57 => {
                let len = byte - 0x40;
                let bytes = self.parse_bytes(len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x58 => {
                let len = self.parse_u8()?;
                let bytes = self.parse_bytes(len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x59 => {
                let len = self.parse_u16()?;
                let bytes = self.parse_bytes(len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x5a => {
                let len = self.parse_u32()?;
                let bytes = self.parse_bytes(len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x5b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let bytes = self.parse_bytes(len as usize)?;
                Ipld::Bytes(bytes)
            }

            // Major type 3: a text string
            0x60..=0x77 => {
                let len = byte - 0x60;
                let string = self.parse_str(len as usize)?;
                Ipld::String(string)
            }
            0x78 => {
                let len = self.parse_u8()?;
                let string = self.parse_str(len as usize)?;
                Ipld::String(string)
            }
            0x79 => {
                let len = self.parse_u16()?;
                let string = self.parse_str(len as usize)?;
                Ipld::String(string)
            }
            0x7a => {
                let len = self.parse_u32()?;
                let string = self.parse_str(len as usize)?;
                Ipld::String(string)
            }
            0x7b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let string = self.parse_str(len as usize)?;
                Ipld::String(string)
            }

            // Major type 4: an array of data items
            0x80..=0x97 => {
                let len = byte - 0x80;
                let list = self.parse_list(len as usize)?;
                Ipld::List(list)
            }
            0x98 => {
                let len = self.parse_u8()?;
                let list = self.parse_list(len as usize)?;
                Ipld::List(list)
            }
            0x99 => {
                let len = self.parse_u16()?;
                let list = self.parse_list(len as usize)?;
                Ipld::List(list)
            }
            0x9a => {
                let len = self.parse_u32()?;
                let list = self.parse_list(len as usize)?;
                Ipld::List(list)
            }
            0x9b => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let list = self.parse_list(len as usize)?;
                Ipld::List(list)
            }

            // Major type 5: a map of pairs of data items
            0xa0..=0xb7 => {
                let len = byte - 0xa0;
                let map = self.parse_map(len as usize)?;
                Ipld::Map(map)
            }
            0xb8 => {
                let len = self.parse_u8()?;
                let map = self.parse_map(len as usize)?;
                Ipld::Map(map)
            }
            0xb9 => {
                let len = self.parse_u16()?;
                let map = self.parse_map(len as usize)?;
                Ipld::Map(map)
            }
            0xba => {
                let len = self.parse_u32()?;
                let map = self.parse_map(len as usize)?;
                Ipld::Map(map)
            }
            0xbb => {
                let len = self.parse_u64()?;
                if len > usize::max_value() as u64 {
                    return Err(CborError::LengthOutOfRange.into());
                }
                let map = self.parse_map(len as usize)?;
                Ipld::Map(map)
            }

            // Major type 6: optional semantic tagging of other major types
            0xd8 => {
                let cid = self.parse_link()?;
                Ipld::Link(cid)
            }

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4 => Ipld::Bool(false),
            0xf5 => Ipld::Bool(true),
            0xf6 => Ipld::Null,
            0xf7 => Ipld::Null,
            0xfa => {
                let value = self.parse_f32()?;
                Ipld::Float(value as f64)
            }
            0xfb => {
                let value = self.parse_f64()?;
                Ipld::Float(value)
            }
            _ => return Err(CborError::UnexpectedCode.into()),
        };
        Ok(ipld)
    }
}
