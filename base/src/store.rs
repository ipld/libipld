use crate::error::BlockError;
use cid::Cid;
use core::future::Future;
use core::pin::Pin;
use std::path::Path;

pub type StoreResult<T> = Pin<Box<dyn Future<Output = Result<T, BlockError>> + Send>>;

/// Visibility of a block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// Block is not announced on the network.
    Private,
    /// Block is announced on the network.
    Public,
}

/// Implementable by ipld storage providers.
pub trait ReadonlyStore {
    /// Returns a block from the store. If the block is not in the
    /// store it fetches it from the network and pins the block. This
    /// future should be wrapped in a timeout. Dropping the future
    /// cancels the request.
    fn get(&self, cid: &Cid) -> StoreResult<Box<[u8]>>;
}

/// Implementable by ipld storage backends.
pub trait Store: ReadonlyStore {
    /// Inserts and pins a block into the store and announces the block
    /// if it is visible.
    fn insert(&self, cid: &Cid, data: Box<[u8]>, visibility: Visibility) -> StoreResult<()>;

    /// Flushes the write buffer.
    fn flush(&self) -> StoreResult<()>;

    /// Marks a block ready for garbage collection.
    fn unpin(&self, cid: &Cid) -> StoreResult<()>;
}

/// Implemented by ipld storage backends that support multiple users.
pub trait MultiUserStore: Store {
    /// Pin a block.
    ///
    /// This creates a symlink chain from root -> path -> block. The block is unpinned by
    /// breaking the symlink chain.
    fn pin(&self, cid: &Cid, path: &Path) -> StoreResult<()>;
}

/// Implemented by ipld storage backends that support aliasing `Cid`s with arbitrary
/// byte strings.
pub trait AliasStore {
    /// Creates an alias for a `Cid` with announces the alias on the public network.
    fn alias(&self, alias: &[u8], cid: &Cid, visibility: Visibility) -> StoreResult<()>;

    /// Removes an alias for a `Cid`.
    fn unalias(&self, alias: &[u8]) -> StoreResult<()>;

    /// Resolves an alias for a `Cid`.
    fn resolve(&self, alias: &[u8]) -> StoreResult<Option<Cid>>;
}
