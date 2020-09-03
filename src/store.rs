//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use core::future::Future;
use core::pin::Pin;

/// Result type of store methods.
pub type StoreResult<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// The status of a block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn pin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move { Err(BlockNotFound(cid.to_string())).into() })
    }

    /// Decreases the pin count of a cid.
    ///
    /// If the block isn't in the store it will return a `BlockNotFound` error.
    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move { Err(BlockNotFound(cid.to_string())).into() })
    }

    /// Returns a block from the store. If the store supports networking and the block is not
    /// in the store it fetches it from the network. Dropping the future cancels the request.
    /// This will not insert the block into the store.
    ///
    /// If the block wasn't found it returns a `BlockNotFound` error.
    fn get<'a>(&'a self, cid: Cid) -> StoreResult<'a, Block<Self::Codec, Self::Multihash>>;

    /// Returns the ipld representation of a block with cid.
    fn get_ipld<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Ipld>
    where
        Ipld: Decode<Self::Codec>,
    {
        Box::pin(async move {
            let block = self.get(cid.clone()).await?;
            block.decode::<Self::Codec, Ipld>()
        })
    }

    /// Resolves a path recursively and returns the ipld.
    fn get_path<'a>(&'a self, path: &'a DagPath<'a>) -> StoreResult<'a, Ipld>
    where
        Ipld: Decode<Self::Codec>,
    {
        Box::pin(async move {
            let mut root = self.get_ipld(path.root()).await?;
            let mut ipld = &root;
            for segment in path.path().iter() {
                ipld = ipld.get(segment)?;
                if let Ipld::Link(cid) = ipld {
                    root = self.get_ipld(cid).await?;
                    ipld = &root;
                }
            }
            Ok(ipld.clone())
        })
    }

    /// Recursively gets a block and all it's references, inserts them into the store and
    /// pins the root.
    ///
    /// If a block wasn't found it returns a `BlockNotFound` error without inserting any blocks
    /// or pinning the root.
    fn sync<'a>(&'a self, cid: Cid) -> StoreResult<'a, Block<Self::Codec, Self::Multihash>> {
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
    fn status<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Status> {
        Box::pin(async move { Ok(Status::new(0, 0)) })
    }

    /// Inserts and pins block into the store and if the store supports networking, it announces
    /// the block on the network.
    ///
    /// If the block is larger than `MAX_BLOCK_SIZE` it returns a `BlockTooLarge` error.
    /// If a block has dangling references it will return a `BlockNotFound` error.
    fn insert<'a>(&'a self, block: &'a Block<Self::Codec, Self::Multihash>) -> StoreResult<'a, ()> {
        self.insert_batch(std::slice::from_ref(block))
    }

    /// Inserts a batch of blocks atomically into the store and pins the last block. If the store
    /// supports networking, it announces all the blocks on the network.
    ///
    /// If a block is larger than `MAX_BLOCK_SIZE` it returns a `BlockTooLarge` error.
    /// If a block has dangling references it will return a `BlockNotFound` error.
    /// If the batch is empty it returns an `EmptyBatch` error.
    fn insert_batch<'a>(
        &'a self,
        batch: &'a [Block<Self::Codec, Self::Multihash>],
    ) -> StoreResult<'a, Cid>;

    /// Flushes the write buffer.
    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }
}

/// Implemented by ipld storage backends that support aliasing `Cid`s with arbitrary
/// byte strings.
pub trait AliasStore: Store {
    /// Creates an alias for a `Cid` with announces the alias on the public network.
    fn alias<'a>(
        &'a self,
        alias: &'a [u8],
        block: &'a Block<Self::Codec, Self::Multihash>,
    ) -> StoreResult<'a, ()>;

    /// Removes an alias for a `Cid`.
    fn unalias<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, ()>;

    /// Resolves an alias for a `Cid`.
    fn resolve<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, Option<Cid>>;
}
