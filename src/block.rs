//! Block
use crate::codec::{Codec, IpldCodec, ToBytes};
use crate::error::Result;
use crate::hash::Hash;
use crate::ipld::Ipld;
pub use cid::Cid;

/// The prefix of a block includes all information to serialize and deserialize
///  to/from ipld.
pub trait Prefix {
    /// The codec to use for encoding ipld.
    type Codec: Codec;
    /// The hash to use to compute the cid.
    type Hash: Hash;
}

/// Block
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    cid: Cid,
    data: Box<[u8]>,
}

impl Block {
    /// Creates a raw block from binary data.
    pub fn new<TPrefix: Prefix>(ipld: &Ipld) -> Result<Self> {
        let data = TPrefix::Codec::to_bytes(ipld)?;
        let hash = TPrefix::Hash::digest(&data);
        let cid = Cid::new_v1(TPrefix::Codec::CODEC, hash);
        Ok(Self { cid, data })
    }

    /// Returns the cid of the block.
    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    /// Splits the block into cid and data.
    pub fn split(self) -> (Cid, Box<[u8]>) {
        (self.cid, self.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::block;

    #[test]
    fn test_block() {
        let block1 = block!({
            "metadata": {
                "type": "file",
                "name": "hello_world.txt",
                "size": 11,
            },
            "content": "hello world",
        })
        .unwrap();
        block!({
            "metadata": {
                "type": "directory",
                "name": "folder",
                "size": 1,
            },
            "children": [ block1.cid() ],
        })
        .unwrap();
    }
}
