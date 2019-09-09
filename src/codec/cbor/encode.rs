//! CBOR encoder.
use crate::error::{format_err, Result};
use crate::ipld::{Ipld, IpldKey, IpldRef};
use byteorder::{BigEndian, ByteOrder};
use cid::Cid;
use core::convert::TryInto;
use std::collections::BTreeMap;
use std::io::Write;

pub struct Encoder<W> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    fn write_u8(&mut self, major: u8, value: u8) -> Result<()> {
        if value <= 0x17 {
            self.writer.write_all(&[major << 5 | value])?;
        } else {
            let buf = [major << 5 | 24, value];
            self.writer.write_all(&buf)?;
        }
        Ok(())
    }

    fn write_u16(&mut self, major: u8, value: u16) -> Result<()> {
        if value <= u16::from(u8::max_value()) {
            self.write_u8(major, value as u8)?;
        } else {
            let mut buf = [major << 5 | 25, 0, 0];
            BigEndian::write_u16(&mut buf[1..], value);
            self.writer.write_all(&buf)?;
        }
        Ok(())
    }

    fn write_u32(&mut self, major: u8, value: u32) -> Result<()> {
        if value <= u32::from(u16::max_value()) {
            self.write_u16(major, value as u16)?;
        } else {
            let mut buf = [major << 5 | 26, 0, 0, 0, 0];
            BigEndian::write_u32(&mut buf[1..], value);
            self.writer.write_all(&buf)?;
        }
        Ok(())
    }

    fn write_u64(&mut self, major: u8, value: u64) -> Result<()> {
        if value <= u64::from(u32::max_value()) {
            self.write_u32(major, value as u32)?;
        } else {
            let mut buf = [major << 5 | 27, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_u64(&mut buf[1..], value);
            self.writer.write_all(&buf)?;
        }
        Ok(())
    }

    fn write_tag(&mut self, tag: u64) -> Result<()> {
        self.write_u64(6, tag)?;
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn write_f32(&mut self, value: f32) -> Result<()> {
        if value.is_infinite() {
            if value.is_sign_positive() {
                self.writer.write_all(&[0xf9, 0x7c, 0x00])?;
            } else {
                self.writer.write_all(&[0xf9, 0xfc, 0x00])?;
            }
        } else if value.is_nan() {
            self.writer.write_all(&[0xf9, 0x7e, 0x00])?;
        } else {
            let mut buf = [0xfa, 0, 0, 0, 0];
            BigEndian::write_f32(&mut buf[1..], value);
            self.writer.write_all(&buf)?;
        }
        Ok(())
    }

    fn encode_bool(&mut self, value: bool) -> Result<()> {
        let value = if value { 0xf5 } else { 0xf4 };
        self.writer.write_all(&[value])?;
        Ok(())
    }

    fn encode_int(&mut self, value: i128) -> Result<()> {
        if value < 0 {
            if -(value + 1) > u64::max_value() as i128 {
                return Err(format_err!("The number can't be stored in CBOR."));
            }
            self.write_u64(1, -(value + 1) as u64)?;
        } else {
            if value > u64::max_value() as i128 {
                return Err(format_err!("The number can't be stored in CBOR."));
            }
            self.write_u64(0, value as u64)?;
        }
        Ok(())
    }
    
    #[allow(clippy::float_cmp)]
    fn encode_float(&mut self, value: f64) -> Result<()> {
        if !value.is_finite() || f64::from(value as f32) == value {
            self.write_f32(value as f32)?;
        } else {
            let mut buf = [0xfb, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_f64(&mut buf[1..], value);
            self.writer.write_all(&buf)?;
        }
        Ok(())
    }

    fn encode_null(&mut self) -> Result<()> {
        self.writer.write_all(&[0xf6])?;
        Ok(())
    }

    fn encode_bytes(&mut self, value: &[u8]) -> Result<()> {
        self.write_u64(2, value.len() as u64)?;
        self.writer.write_all(value)?;
        Ok(())
    }

    fn encode_str(&mut self, value: &str) -> Result<()> {
        self.write_u64(3, value.len() as u64)?;
        self.writer.write_all(value.as_bytes())?;
        Ok(())
    }

    fn encode_list(&mut self, value: &[Ipld]) -> Result<()> {
        self.write_u64(4, value.len().try_into()?)?;
        for ipld in value {
            self.encode_ipld(ipld)?;
        }
        Ok(())
    }

    fn encode_list_ref<'a>(&mut self, value: &[IpldRef<'a>]) -> Result<()> {
        self.write_u64(4, value.len().try_into()?)?;
        for ipld in value {
            self.encode_ipld_ref(ipld)?;
        }
        Ok(())
    }

    fn encode_map(&mut self, value: &BTreeMap<IpldKey, Ipld>) -> Result<()> {
        self.write_u64(5, value.len().try_into()?)?;
        for (k, v) in value {
            self.encode_key(k)?;
            self.encode_ipld(v)?;
        }
        Ok(())
    }

    fn encode_map_ref<'a>(&mut self, value: &BTreeMap<IpldKey, IpldRef<'a>>) -> Result<()> {
        self.write_u64(5, value.len().try_into()?)?;
        for (k, v) in value {
            self.encode_key(k)?;
            self.encode_ipld_ref(v)?;
        }
        Ok(())
    }

    fn encode_key(&mut self, ipld: &IpldKey) -> Result<()> {
        match ipld {
            IpldKey::Integer(i) => self.encode_int(*i),
            IpldKey::Bytes(b) => self.encode_bytes(b),
            IpldKey::String(s) => self.encode_str(s),
        }
    }

    fn encode_link(&mut self, cid: &Cid) -> Result<()> {
        self.write_tag(42)?;
        self.encode_bytes(&cid.to_bytes())?;
        Ok(())
    }

    fn encode_ipld(&mut self, ipld: &Ipld) -> Result<()> {
        match ipld {
            Ipld::Null => self.encode_null(),
            Ipld::Bool(b) => self.encode_bool(*b),
            Ipld::Integer(i) => self.encode_int(*i),
            Ipld::Float(f) => self.encode_float(*f),
            Ipld::Bytes(b) => self.encode_bytes(b),
            Ipld::String(s) => self.encode_str(s),
            Ipld::List(l) => self.encode_list(l),
            Ipld::Map(m) => self.encode_map(m),
            Ipld::Link(c) => self.encode_link(c),
        }
    }

    fn encode_ipld_ref<'a>(&mut self, ipld: &IpldRef<'a>) -> Result<()> {
        match ipld {
            IpldRef::Null => self.encode_null(),
            IpldRef::Bool(b) => self.encode_bool(*b),
            IpldRef::Integer(i) => self.encode_int(*i),
            IpldRef::Float(f) => self.encode_float(*f),
            IpldRef::Bytes(b) => self.encode_bytes(b),
            IpldRef::String(s) => self.encode_str(s),
            IpldRef::List(l) => self.encode_list(l),
            IpldRef::OwnedList(l) => self.encode_list_ref(l),
            IpldRef::Map(m) => self.encode_map(m),
            IpldRef::OwnedMap(m) => self.encode_map_ref(m),
            IpldRef::Link(c) => self.encode_link(c),
        }
    }

    pub fn encode<'a>(&mut self, ipld: IpldRef<'a>) -> Result<()> {
        self.encode_ipld_ref(&ipld)
    }
}

