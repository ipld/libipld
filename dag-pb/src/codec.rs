use crate::dag_pb;
use core::convert::{TryFrom, TryInto};
use libipld_core::cid::Cid;
use libipld_core::error::{Result, TypeError, TypeErrorType};
use libipld_core::ipld::Ipld;
use prost::bytes::{Buf, Bytes};
use std::collections::BTreeMap;

/// A protobuf ipld link.
#[derive(Debug)]
pub struct PbLink {
    /// Content identifier.
    pub cid: Cid,
    /// Name of the link.
    pub name: String,
    /// Size of the data.
    pub size: u64,
}

/// A protobuf ipld node.
#[derive(Debug)]
pub struct PbNode {
    /// List of protobuf ipld links.
    pub links: Vec<PbLink>,
    /// Binary data blob.
    pub data: Box<[u8]>,
}

use prost::Message;

impl PbNode {
    pub(crate) fn links(bytes: Bytes, links: &mut impl Extend<Cid>) -> Result<()> {
        let proto = dag_pb::PbNode::decode(bytes)?;
        for link in proto.links {
            let cid = Cid::try_from(link.hash.as_ref())?;
            links.extend(Some(cid));
        }
        Ok(())
    }

    /// Deserializes a `PbNode` from bytes.
    pub fn from_bytes(bytes: impl Buf) -> Result<Self> {
        let proto: dag_pb::PbNode = dag_pb::PbNode::decode(bytes)?;
        let data = proto.data.to_vec().into_boxed_slice();
        let mut links = Vec::new();
        for link in proto.links {
            let cid = Cid::try_from(link.hash.as_ref())?;
            let name = link.name;
            let size = link.tsize;
            links.push(PbLink { cid, name, size });
        }
        Ok(PbNode { links, data })
    }

    /// Serializes a `PbNode` to bytes.
    pub fn into_bytes(self) -> Box<[u8]> {
        let links = self
            .links
            .into_iter()
            .map(|link| dag_pb::PbLink {
                hash: link.cid.to_bytes().into(),
                name: link.name,
                tsize: link.size,
            })
            .collect::<Vec<_>>();
        let proto = dag_pb::PbNode {
            data: self.data.into(),
            links,
        };

        let mut res = Vec::with_capacity(proto.encoded_len());
        proto
            .encode(&mut res)
            .expect("there is no situation in which the protobuf message can be invalid");
        res.into_boxed_slice()
    }
}

impl From<PbNode> for Ipld {
    fn from(node: PbNode) -> Self {
        let mut map = BTreeMap::<String, Ipld>::new();
        let links = node
            .links
            .into_iter()
            .map(|link| link.into())
            .collect::<Vec<Ipld>>();
        map.insert("Links".to_string(), links.into());
        map.insert("Data".to_string(), node.data.into());
        map.into()
    }
}

impl From<PbLink> for Ipld {
    fn from(link: PbLink) -> Self {
        let mut map = BTreeMap::<String, Ipld>::new();
        map.insert("Hash".to_string(), link.cid.into());
        map.insert("Name".to_string(), link.name.into());
        map.insert("Tsize".to_string(), link.size.into());
        map.into()
    }
}

impl TryFrom<&Ipld> for PbNode {
    type Error = TypeError;

    fn try_from(ipld: &Ipld) -> core::result::Result<PbNode, Self::Error> {
        let links = if let Ipld::List(links) = ipld.get("Links")? {
            links
                .iter()
                .map(|link| link.try_into())
                .collect::<Result<_, _>>()?
        } else {
            return Err(TypeError::new(TypeErrorType::List, ipld));
        };
        let data = if let Ipld::Bytes(data) = ipld.get("Data")? {
            data.clone().into_boxed_slice()
        } else {
            return Err(TypeError::new(TypeErrorType::Bytes, ipld));
        };
        Ok(PbNode { links, data })
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
        let name = if let Ipld::String(name) = ipld.get("Name")? {
            name.clone()
        } else {
            return Err(TypeError::new(TypeErrorType::String, ipld));
        };
        let size = if let Ipld::Integer(size) = ipld.get("Tsize")? {
            *size as u64
        } else {
            return Err(TypeError::new(TypeErrorType::Integer, ipld));
        };
        Ok(PbLink { cid, name, size })
    }
}
