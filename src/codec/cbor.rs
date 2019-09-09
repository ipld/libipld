//! CBOR codec.
use crate::codec::Codec;
use crate::error::{format_err, Result};
use crate::ipld::{Ipld, IpldKey, IpldRef};
use byteorder::{BigEndian, ByteOrder};
use cid::Cid;
use core::convert::TryInto;
use std::collections::BTreeMap;
use std::io::Write;

/// CBOR codec.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DagCbor;

struct Encoder<W> {
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

    fn encode_null(&mut self) -> Result<()> {
        self.writer.write_all(&[0xf6])?;
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

    fn encode_float(&mut self, _value: f64) -> Result<()> {
        unimplemented!()
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

/*
fn decode(cbor: &Value) -> Result<Ipld> {
    let ipld = match cbor {
        Value::Null => Ipld::Null,
        Value::Bool(b) => Ipld::Bool(*b),
        Value::Integer(i) => Ipld::Integer(*i),
        Value::Float(f) => Ipld::Float(*f),
        Value::Bytes(bytes) => Ipld::Bytes(bytes.to_owned()),
        Value::Text(string) => Ipld::String(string.to_owned()),
        Value::Array(array) => {
            let mut list = Vec::with_capacity(array.len());
            for item in array.iter() {
                list.push(decode(item)?);
            }
            Ipld::List(list)
        }
        Value::Map(object) => {
            if let Some(Value::Bytes(bytes)) = object.get(&Value::Integer(42)) {
                Ipld::Link(Cid::try_from(bytes.as_slice())?)
            } else {
                let mut map = HashMap::with_capacity(object.len());
                for (k, v) in object.iter() {
                    map.insert(decode(k)?.try_into()?, decode(v)?);
                }
                Ipld::Map(map)
            }
        }
        Value::__Hidden => return Err(format_err!("__Hidden value not supported")),
    };
    Ok(ipld)
}*/

impl Codec for DagCbor {
    const VERSION: cid::Version = cid::Version::V1;
    const CODEC: cid::Codec = cid::Codec::DagCBOR;

    fn encode<'a>(ipld: IpldRef<'a>) -> Result<Box<[u8]>> {
        let mut bytes = Vec::new();
        let mut enc = Encoder::new(&mut bytes);
        enc.encode(ipld)?;
        Ok(bytes.into_boxed_slice())
    }

    fn decode(_data: &[u8]) -> Result<Ipld> {
        Ok(Ipld::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block, ipld};

    #[test]
    fn encode_decode_cbor() {
        let link = block!(null).unwrap();
        let ipld = ipld!({
          "number": 1,
          "list": [true, null],
          "bytes": vec![0, 1, 2, 3],
          "link": link.cid(),
        });
        let ipld2 = DagCbor::decode(&DagCbor::encode(ipld.as_ref()).unwrap()).unwrap();
        assert_eq!(ipld, ipld2);
    }
}
