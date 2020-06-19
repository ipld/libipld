use crate::Error;
use core::convert::{TryFrom, TryInto};
use libipld_core::cid::CidGeneric;
use libipld_core::error::{TypeError, TypeErrorType};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

mod dag_pb {
    include!(concat!(env!("OUT_DIR"), "/dag_pb.rs"));
}

/// A protobuf ipld link.
pub struct PbLink<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    /// Content identifier.
    pub cid: CidGeneric<u64, H>,
    /// Name of the link.
    pub name: String,
    /// Size of the data.
    pub size: u64,
}

/// A protobuf ipld node.
pub struct PbNode<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    /// List of protobuf ipld links.
    pub links: Vec<PbLink<H>>,
    /// Binary data blob.
    pub data: Box<[u8]>,
}

use prost::Message;

impl<H> PbNode<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    /// Deserializes a `PbNode` from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let proto: dag_pb::PbNode = dag_pb::PbNode::decode(bytes)?;
        let data = proto.data.into_boxed_slice();
        let mut links = Vec::new();
        for link in proto.links {
            let cid = CidGeneric::<u64, H>::try_from(link.hash)?;
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

impl<H> Into<Ipld<H>> for PbNode<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn into(self) -> Ipld<H> {
        let mut map = BTreeMap::<String, Ipld<H>>::new();
        let links = self
            .links
            .into_iter()
            .map(|link| link.into())
            .collect::<Vec<Ipld<H>>>();
        map.insert("Links".to_string(), links.into());
        map.insert("Data".to_string(), self.data.into());
        map.into()
    }
}

impl<H> Into<Ipld<H>> for PbLink<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    fn into(self) -> Ipld<H> {
        let mut map = BTreeMap::<String, Ipld<H>>::new();
        map.insert("Hash".to_string(), self.cid.into());
        map.insert("Name".to_string(), self.name.into());
        map.insert("Tsize".to_string(), self.size.into());
        map.into()
    }
}

impl<H> TryFrom<&Ipld<H>> for PbNode<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    type Error = TypeError;

    fn try_from(ipld: &Ipld<H>) -> Result<PbNode<H>, Self::Error> {
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

impl<H> TryFrom<&Ipld<H>> for PbLink<H>
where
    H: Into<u64> + TryFrom<u64> + Copy,
{
    type Error = TypeError;

    fn try_from(ipld: &Ipld<H>) -> Result<PbLink<H>, Self::Error> {
        let cid = if let Ipld::Link(cid) = ipld.get("Hash")? {
            cid.clone()
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
