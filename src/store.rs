//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use async_trait::async_trait;
use std::collections::HashSet;

/// The status of a block.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Status {
    pinned: usize,
    referenced: usize,
}

impl Status {
    /// Creates a new status.
    pub fn new(pinned: usize, referenced: usize) -> Self {
        Self { pinned, referenced }
    }

    /// Returns the number of times the block is pinned.
    pub fn pinned(&self) -> usize {
        self.pinned
    }

    /// Returns the number of references to the block.
    pub fn referenced(&self) -> usize {
        self.referenced
    }

    /// The block is pinned at least once.
    pub fn is_pinned(&self) -> bool {
        self.pinned > 0
    }

    /// The block is referenced at least once.
    pub fn is_referenced(&self) -> bool {
        self.referenced > 0
    }

    /// The block is not going to be garbage collected.
    pub fn is_live(&self) -> bool {
        self.is_pinned() || self.is_referenced()
    }

    /// The block is going to be garbage collected.
    pub fn is_dead(&self) -> bool {
        self.pinned == 0 && self.referenced == 0
    }
}

/// Implementable by ipld stores.
#[async_trait]
pub trait Store: Clone + Send + Sync {
    /// The multihash type of the store.
    type Multihash: MultihashDigest;
    /// The codec type of the store.
    type Codec: Codec;
    /// The maximum block size supported by the store.
    const MAX_BLOCK_SIZE: usize;

    /// Increases the pin count on a cid.
    ///
    /// If the block isn't in the store it will return a `BlockNotFound` error.
    async fn pin(&self, cid: &Cid) -> Result<()>;

    /// Decreases the pin count of a cid.
    ///
    /// If the block isn't in the store it will return a `BlockNotFound` error.
    async fn unpin(&self, cid: &Cid) -> Result<()>;

    /// Returns a block from the store. If the store supports networking and the block is not
    /// in the store it fetches it from the network. Dropping the future cancels the request.
    /// This will not insert the block into the store.
    ///
    /// If the block wasn't found it returns a `BlockNotFound` error.
    async fn get(&self, cid: Cid) -> Result<Block<Self::Codec, Self::Multihash>>;

    /// Resolves a path recursively and returns the ipld.
    async fn query(&self, path: &DagPath<'_>) -> Result<Ipld>
    where
        Ipld: Decode<Self::Codec>,
    {
        let mut root = self.get(path.root().clone()).await?.ipld()?;
        let mut ipld = &root;
        for segment in path.path().iter() {
            ipld = ipld.get(segment)?;
            if let Ipld::Link(cid) = ipld {
                root = self.get(cid.clone()).await?.ipld()?;
                ipld = &root;
            }
        }
        Ok(ipld.clone())
    }

    /// Recursively gets a block and all it's references, inserts them into the store and
    /// pins the root.
    ///
    /// If a block wasn't found it returns a `BlockNotFound` error without inserting any blocks
    /// or pinning the root.
    async fn sync(&self, cid: Cid) -> Result<Block<Self::Codec, Self::Multihash>>
    where
        Ipld: Decode<Self::Codec>,
    {
        let mut visited = HashSet::new();
        let mut blocks = vec![];
        let mut stack = vec![cid];
        while let Some(cid) = stack.pop() {
            if visited.contains(&cid) {
                continue;
            }
            let block = self.get(cid).await?;
            for r in block.references()? {
                stack.push(r);
            }
            visited.insert(block.cid.clone());
            blocks.push(block);
        }
        blocks.reverse();
        let cid = self.insert_batch(&blocks).await?;
        self.get(cid).await
    }

    /// Returns the status of a block.
    async fn status(&self, cid: &Cid) -> Result<Option<Status>>;

    /// Inserts and pins block into the store and if the store supports networking, it announces
    /// the block on the network.
    ///
    /// If the block is larger than `MAX_BLOCK_SIZE` it returns a `BlockTooLarge` error.
    /// If a block has dangling references it will return a `BlockNotFound` error.
    async fn insert(&self, block: &Block<Self::Codec, Self::Multihash>) -> Result<Cid> {
        self.insert_batch(std::slice::from_ref(block)).await
    }

    /// Inserts a batch of blocks atomically into the store and pins the last block. If the store
    /// supports networking, it announces all the blocks on the network.
    ///
    /// If a block is larger than `MAX_BLOCK_SIZE` it returns a `BlockTooLarge` error.
    /// If a block has dangling references it will return a `BlockNotFound` error.
    /// If the batch is empty it returns an `EmptyBatch` error.
    async fn insert_batch(&self, batch: &[Block<Self::Codec, Self::Multihash>]) -> Result<Cid>;

    /// Flushes the write buffer.
    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// Implemented by ipld storage backends that support aliasing `Cid`s with arbitrary
/// byte strings.
#[async_trait]
pub trait AliasStore: Store {
    /// Creates an alias for a `Cid` with announces the alias on the public network.
    async fn alias(&self, alias: &[u8], cid: &Cid) -> Result<()>;

    /// Removes an alias for a `Cid`.
    async fn unalias(&self, alias: &[u8]) -> Result<()>;

    /// Resolves an alias for a `Cid`.
    async fn resolve(&self, alias: &[u8]) -> Result<Option<Cid>>;
}
