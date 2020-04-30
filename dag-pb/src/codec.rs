use crate::ProtobufError as Error;
use core::convert::{TryFrom, TryInto};
use libipld_core::cid::Cid;
use libipld_core::error::IpldError;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

mod dag_pb {
    include!(concat!(env!("OUT_DIR"), "/dag_pb.rs"));
}

pub struct PbLink {
    pub cid: Cid,
    pub name: String,
    pub size: u64,
}

pub struct PbNode {
    pub links: Vec<PbLink>,
    pub data: Vec<u8>,
}

use prost::Message;

impl PbNode {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let proto: dag_pb::PbNode = dag_pb::PbNode::decode(bytes)?;
        let data = proto.data;
        let mut links = Vec::new();
        for link in proto.links {
            let cid = Cid::try_from(link.hash)?;
            let name = link.name;
            let size = link.tsize;
            links.push(PbLink { cid, name, size });
        }
        Ok(PbNode { links, data })
    }

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
            data: self.data,
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
    type Error = IpldError;

    fn try_from(ipld: &Ipld) -> Result<PbNode, Self::Error> {
        let links = if let Ipld::List(links) = ipld.get("Links").ok_or(IpldError::KeyNotFound)? {
            links
                .iter()
                .map(|link| link.try_into())
                .collect::<Result<_, _>>()?
        } else {
            return Err(IpldError::NotList);
        };
        let data = if let Ipld::Bytes(data) = ipld.get("Data").ok_or(IpldError::KeyNotFound)? {
            data.clone()
        } else {
            return Err(IpldError::NotBytes);
        };
        Ok(PbNode { links, data })
    }
}

impl TryFrom<&Ipld> for PbLink {
    type Error = IpldError;

    fn try_from(ipld: &Ipld) -> Result<PbLink, Self::Error> {
        let cid = if let Ipld::Link(cid) = ipld.get("Hash").ok_or(IpldError::KeyNotFound)? {
            cid.clone()
        } else {
            return Err(IpldError::NotLink);
        };
        let name = if let Ipld::String(name) = ipld.get("Name").ok_or(IpldError::KeyNotFound)? {
            name.clone()
        } else {
            return Err(IpldError::NotString);
        };
        let size = if let Ipld::Integer(size) = ipld.get("Tsize").ok_or(IpldError::KeyNotFound)? {
            *size as u64
        } else {
            return Err(IpldError::NotInteger);
        };
        Ok(PbLink { cid, name, size })
    }
}
