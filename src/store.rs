//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::error::StoreError;
use core::future::Future;
use core::pin::Pin;
use std::path::Path;

/// Result type of store methods.
pub type StoreResult<'a, T> = Pin<Box<dyn Future<Output = Result<T, StoreError>> + Send + 'a>>;

/// Visibility of a block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// Block is not announced on the network.
    Private,
    /// Block is announced on the network.
    Public,
}

/// Implementable by ipld storage providers.
pub trait ReadonlyStore: Clone {
    /// Returns a block from the store. If the block is not in the store it fetches it from the
    /// network and pins the block. Dropping the future cancels the request.
    fn get<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Box<[u8]>>;
}

/// Implementable by ipld storage backends.
pub trait Store: ReadonlyStore {
    /// Inserts and pins block into the store and announces it if it is visible.
    fn insert<'a>(
        &'a self,
        cid: &'a Cid,
        data: Box<[u8]>,
        visibility: Visibility,
    ) -> StoreResult<'a, ()>;

    /// Inserts a batch of blocks atomically into the store and announces them block
    /// if it is visible. The last block is pinned.
    fn insert_batch<'a>(
        &'a self,
        batch: Vec<Block>,
        visibility: Visibility,
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
pub trait AliasStore {
    /// Creates an alias for a `Cid` with announces the alias on the public network.
    fn alias<'a>(
        &'a self,
        alias: &'a [u8],
        cid: &'a Cid,
        visibility: Visibility,
    ) -> StoreResult<'a, ()>;

    /// Removes an alias for a `Cid`.
    fn unalias<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, ()>;

    /// Resolves an alias for a `Cid`.
    fn resolve<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, Option<Cid>>;
}
