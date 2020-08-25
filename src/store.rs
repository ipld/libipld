//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use core::future::Future;
use core::pin::Pin;
use std::path::Path;

/// Result type of store methods.
pub type StoreResult<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Implementable by ipld storage providers.
pub trait ReadonlyStore: Clone + Send + Sync {
    /// The multihash type of the store.
    type Multihash: MultihashDigest;
    /// The codec type of the store.
    type Codec: Codec;
    /// The maximum block size supported by the store.
    const MAX_BLOCK_SIZE: usize;

    /// Returns a block from the store. If the block is not in the store it fetches it from the
    /// network and pins the block. Dropping the future cancels the request.
    ///
    /// If the block wasn't found it returns a `BlockNotFound` error.
    fn get<'a>(&'a self, cid: Cid) -> StoreResult<'a, Block<Self::Codec, Self::Multihash>>;

    /// Returns the ipld representation of a block with cid.
    fn get_ipld<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Ipld> {
        Box::pin(async move {
            let block = self.get(cid.clone()).await?;
            block.decode_ipld()
        })
    }

    /// Resolves a path recursively and returns the ipld.
    fn get_path<'a>(&'a self, path: &'a DagPath<'a>) -> StoreResult<'a, Ipld> {
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
}

/// Implementable by ipld storage backends.
pub trait Store: ReadonlyStore {
    /// Inserts and pins block into the store and announces it if it is visible.
    ///
    /// If the block is larger than `MAX_BLOCK_SIZE` it returns a `BlockTooLarge` error.
    fn insert<'a>(&'a self, block: &'a Block<Self::Codec, Self::Multihash>) -> StoreResult<'a, ()>;

    /// Inserts a batch of blocks atomically into the store and announces them block
    /// if it is visible. The last block is pinned.
    ///
    /// If the block is larger than `MAX_BLOCK_SIZE` it returns a `BlockTooLarge` error.
    /// If the batch is empty it returns an `EmptyBatch` error.
    fn insert_batch<'a>(
        &'a self,
        batch: &'a [Block<Self::Codec, Self::Multihash>],
    ) -> StoreResult<'a, Cid>;

    /// Flushes the write buffer.
    fn flush(&self) -> StoreResult<'_, ()>;

    /// Decreases the ref count on a cid.
    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()>;
}

/// Implemented by ipld storage backends that support multiple users.
pub trait MultiUserStore: Store {
    /// Pin a block.
    ///
    /// This creates a symlink chain from root -> path -> block. The block is unpinned by
    /// breaking the symlink chain.
    fn pin<'a>(&'a self, cid: &'a Cid, path: &'a Path) -> StoreResult<'a, ()>;
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
