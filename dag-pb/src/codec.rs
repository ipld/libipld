use core::convert::{TryFrom, TryInto};
use std::borrow::Cow;
use std::collections::BTreeMap;

use libipld_core::cid::Cid;
use libipld_core::error::{Result, TypeError, TypeErrorType};
use libipld_core::ipld::Ipld;
use quick_protobuf::sizeofs::*;
use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer, WriterBackend};

/// A protobuf ipld link.
#[derive(Debug)]
pub struct PbLink {
    /// Content identifier.
    pub cid: Cid,
    /// Name of the link.
    pub name: Option<String>,
    /// Size of the data.
    pub size: Option<u64>,
}

/// A protobuf ipld node.
#[derive(Debug, Default)]
pub struct PbNode<'a> {
    /// List of protobuf ipld links.
    pub links: Vec<PbLink>,
    /// Binary data blob.
    pub data: Option<Cow<'a, [u8]>>,
}

impl<'a> PbNode<'a> {
    pub(crate) fn links(bytes: &[u8], links: &mut impl Extend<Cid>) -> Result<()> {
        let node = PbNode::from_bytes(bytes)?;
        for link in node.links {
            links.extend(Some(link.cid));
        }
        Ok(())
    }

    /// Deserializes a `PbNode` from bytes.
    pub fn from_bytes(buf: &'a [u8]) -> Result<Self> {
        // For compat reasons, an empty buf is equal to the default.
        let mut reader = BytesReader::from_bytes(buf);
        let node = Self::from_reader(&mut reader, buf)?;
        Ok(node)
    }

    /// Serializes a `PbNode` to bytes.
    pub fn into_bytes(mut self) -> Box<[u8]> {
        // Links must be strictly sorted by name before encoding, leaving stable
        // ordering where the names are the same (or absent).
        self.links.sort_by(|a, b| {
            let a = a.name.as_ref().map(|s| s.as_bytes()).unwrap_or(&[][..]);
            let b = b.name.as_ref().map(|s| s.as_bytes()).unwrap_or(&[][..]);
            a.cmp(b)
        });

        let mut buf = Vec::with_capacity(self.get_size());
        let mut writer = Writer::new(&mut buf);
        self.write_message(&mut writer)
            .expect("protobuf to be valid");
        buf.into_boxed_slice()
    }
}

impl<'a> From<PbNode<'a>> for Ipld {
    fn from(node: PbNode) -> Self {
        let mut map = BTreeMap::<String, Ipld>::new();
        let links = node
            .links
            .into_iter()
            .map(|link| link.into())
            .collect::<Vec<Ipld>>();
        map.insert("Links".to_string(), links.into());
        if let Some(data) = node.data {
            map.insert("Data".to_string(), Ipld::Bytes(data.to_vec()));
        }
        map.into()
    }
}

impl From<PbLink> for Ipld {
    fn from(link: PbLink) -> Self {
        let mut map = BTreeMap::<String, Ipld>::new();
        map.insert("Hash".to_string(), link.cid.into());

        if let Some(name) = link.name {
            map.insert("Name".to_string(), name.into());
        }
        if let Some(size) = link.size {
            map.insert("Tsize".to_string(), size.into());
        }
        map.into()
    }
}

impl<'a> TryFrom<&'a Ipld> for PbNode<'a> {
    type Error = TypeError;

    fn try_from(ipld: &'a Ipld) -> core::result::Result<PbNode, Self::Error> {
        let mut node = PbNode::default();

        if let Ipld::List(links) = ipld.get("Links")? {
            node.links = links
                .iter()
                .map(|link| link.try_into())
                .collect::<Result<_, _>>()?
        }
        if let Ok(Ipld::Bytes(data)) = ipld.get("Data") {
            node.data = Some(Cow::Borrowed(&data[..]));
        }

        Ok(node)
    }
}

impl TryFrom<&Ipld> for PbLink {
    type Error = TypeError;

    fn try_from(ipld: &Ipld) -> core::result::Result<PbLink, Self::Error> {
        let cid = if let Ipld::Link(cid) = ipld.get("Hash")? {
            *cid
        } else {
            return Err(TypeError::new(TypeErrorType::Link, ipld));
        };

        let mut link = PbLink {
            cid,
            name: None,
            size: None,
        };

        if let Ok(Ipld::String(name)) = ipld.get("Name") {
            link.name = Some(name.clone());
        }
        if let Ok(Ipld::Integer(size)) = ipld.get("Tsize") {
            link.size = Some(*size as u64);
        }

        Ok(link)
    }
}

impl<'a> MessageRead<'a> for PbLink {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> quick_protobuf::Result<Self> {
        let mut cid = None;
        let mut name = None;
        let mut size = None;

        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => {
                    let bytes = r.read_bytes(bytes)?;
                    cid = Some(
                        Cid::try_from(bytes)
                            .map_err(|e| quick_protobuf::Error::Message(e.to_string()))?,
                    );
                }
                Ok(18) => name = Some(r.read_string(bytes)?.to_string()),
                Ok(24) => size = Some(r.read_uint64(bytes)?),
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(PbLink {
            cid: cid.ok_or_else(|| quick_protobuf::Error::Message("missing Hash".into()))?,
            name,
            size,
        })
    }
}

impl MessageWrite for PbLink {
    fn get_size(&self) -> usize {
        let mut size = 0;
        let l = self.cid.encoded_len();
        size += 1 + sizeof_len(l);

        if let Some(ref name) = self.name {
            size += 1 + sizeof_len(name.as_bytes().len());
        }

        if let Some(tsize) = self.size {
            size += 1 + sizeof_varint(tsize as u64);
        }
        size
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        let bytes = self.cid.to_bytes();
        w.write_with_tag(10, |w| w.write_bytes(&bytes))?;

        if let Some(ref name) = self.name {
            w.write_with_tag(18, |w| w.write_string(name))?;
        }
        if let Some(size) = self.size {
            w.write_with_tag(24, |w| w.write_uint64(size))?;
        }
        Ok(())
    }
}

impl<'a> MessageRead<'a> for PbNode<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> quick_protobuf::Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(18) => msg.links.push(r.read_message::<PbLink>(bytes)?),
                Ok(10) => msg.data = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PbNode<'a> {
    fn get_size(&self) -> usize {
        let mut size = 0;
        if let Some(ref data) = self.data {
            size += 1 + sizeof_len(data.len());
        }

        size += self
            .links
            .iter()
            .map(|s| 1 + sizeof_len((s).get_size()))
            .sum::<usize>();

        size
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> quick_protobuf::Result<()> {
        for s in &self.links {
            w.write_with_tag(18, |w| w.write_message(s))?;
        }

        if let Some(ref data) = self.data {
            w.write_with_tag(10, |w| w.write_bytes(data))?;
        }

        Ok(())
    }
}
