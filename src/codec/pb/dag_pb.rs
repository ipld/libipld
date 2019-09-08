use super::gen;
use crate::error::{format_err, Error, Result};
use crate::ipld;
use crate::ipld::Ipld;
use cid::Cid;
use protobuf::Message;
use std::convert::{TryFrom, TryInto};

#[derive(Clone)]
pub struct PbLink {
    pub cid: Cid,
    pub name: String,
    pub size: u64,
}

impl Into<Ipld> for PbLink {
    fn into(self) -> Ipld {
        ipld!({
            "Hash": self.cid,
            "Name": self.name,
            "Tsize": self.size,
        })
    }
}

fn from_ipld(ipld: Ipld) -> Option<PbLink> {
    if let Ipld::Map(mut map) = ipld {
        let cid: Option<Cid> = map
            .remove(&"Hash".into())
            .map(|t| TryInto::try_into(t).ok())
            .unwrap_or_default();
        let name: Option<String> = map
            .remove(&"Name".into())
            .map(|t| TryInto::try_into(t).ok())
            .unwrap_or_default();
        let size: Option<u64> = map
            .remove(&"Tsize".into())
            .map(|t| TryInto::try_into(t).ok())
            .unwrap_or_default();
        if let (Some(cid), Some(name), Some(size)) = (cid, name, size) {
            return Some(PbLink { cid, name, size });
        }
    }
    None
}

#[derive(Clone, Default)]
pub struct PbNode {
    pub links: Vec<PbLink>,
    pub data: Vec<u8>,
}

impl Into<Ipld> for PbNode {
    fn into(self) -> Ipld {
        let links: Vec<Ipld> = self.links.into_iter().map(Into::into).collect();
        ipld!({
            "Links": links,
            "Data": self.data,
        })
    }
}

impl TryFrom<&Ipld> for PbNode {
    type Error = Error;

    fn try_from(ipld: &Ipld) -> Result<Self> {
        match ipld {
            Ipld::Map(map) => {
                let links: Vec<Ipld> = map
                    .get(&"Links".into())
                    .cloned()
                    .map(|t| TryInto::try_into(t).ok())
                    .unwrap_or_default()
                    .unwrap_or_default();
                let links: Vec<PbLink> = links.into_iter().filter_map(from_ipld).collect();
                let data: Vec<u8> = map
                    .get(&"Data".into())
                    .cloned()
                    .map(|t| TryInto::try_into(t).ok())
                    .unwrap_or_default()
                    .unwrap_or_default();
                Ok(PbNode { links, data })
            }
            _ => Err(format_err!("Expected map")),
        }
    }
}

impl PbNode {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let proto: gen::PBNode = protobuf::parse_from_bytes(bytes)?;
        let data = proto.get_Data().to_vec();
        let mut links = Vec::new();
        for link in proto.get_Links() {
            let cid = Cid::try_from(link.get_Hash())?;
            let name = link.get_Name().to_string();
            let size = link.get_Tsize();
            links.push(PbLink { cid, name, size });
        }
        Ok(PbNode { links, data })
    }

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let mut proto = gen::PBNode::new();
        proto.set_Data(self.data);
        for link in self.links {
            let mut pb_link = gen::PBLink::new();
            pb_link.set_Hash(link.cid.to_bytes());
            pb_link.set_Name(link.name);
            pb_link.set_Tsize(link.size);
            proto.mut_Links().push(pb_link);
        }
        Ok(proto.write_to_bytes()?)
    }
}
