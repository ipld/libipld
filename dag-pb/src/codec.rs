use core::convert::{TryFrom, TryInto};
use libipld_core::cid::Cid;
use libipld_core::error::{Result, TypeError, TypeErrorType};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

mod dag_pb {
    include!(concat!(env!("OUT_DIR"), "/dag_pb.rs"));
}

/// A protobuf ipld link.
pub struct PbLink {
    /// Content identifier.
    pub cid: Cid,
    /// Name of the link.
    pub name: String,
    /// Size of the data.
    pub size: u64,
}

/// A protobuf ipld node.
pub struct PbNode {
    /// List of protobuf ipld links.
    pub links: Vec<PbLink>,
    /// Binary data blob.
    pub data: Box<[u8]>,
}

use prost::Message;

impl PbNode {
    /// Deserializes a `PbNode` from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let proto: dag_pb::PbNode = dag_pb::PbNode::decode(bytes)?;
        let data = proto.data.into_boxed_slice();
        let mut links = Vec::new();
        for link in proto.links {
            let cid = Cid::try_from(link.hash)?;
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
                hash: link.cid.to_bytes(),
                name: link.name,
                tsize: link.size,
            })
            .collect::<Vec<_>>();
        let proto = dag_pb::PbNode {
            data: self.data.into_vec(),
            links,
        };

        let mut res = Vec::with_capacity(proto.encoded_len());
        proto
            .encode(&mut res)
            .expect("there is no situation in which the protobuf message can be invalid");
        res.into_boxed_slice()
    }
}

impl Into<Ipld> for PbNode {
    fn into(self) -> Ipld {
        let mut map = BTreeMap::<String, Ipld>::new();
        let links = self
            .links
            .into_iter()
            .map(|link| link.into())
            .collect::<Vec<Ipld>>();
        map.insert("Links".to_string(), links.into());
        map.insert("Data".to_string(), self.data.into());
        map.into()
    }
}

impl Into<Ipld> for PbLink {
    fn into(self) -> Ipld {
        let mut map = BTreeMap::<String, Ipld>::new();
        map.insert("Hash".to_string(), self.cid.into());
        map.insert("Name".to_string(), self.name.into());
        map.insert("Tsize".to_string(), self.size.into());
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
