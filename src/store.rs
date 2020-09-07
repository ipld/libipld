//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use async_trait::async_trait;
use std::collections::HashSet;

/// The status of a block.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Status {
    pinned: u32,
    referenced: u32,
}

impl Status {
    /// Creates a new status.
    pub fn new(pinned: u32, referenced: u32) -> Self {
        Self { pinned, referenced }
    }

    /// Returns the number of times the block is pinned.
    pub fn pinned(&self) -> u32 {
        self.pinned
    }

    /// Returns the number of references to the block.
    pub fn referenced(&self) -> u32 {
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
        self.pinned < 1 && self.referenced < 1
    }

    /// Pin.
    pub fn pin(&mut self) {
        self.pinned += 1;
    }

    /// Unpin.
    pub fn unpin(&mut self) {
        if self.is_pinned() {
            self.pinned -= 1;
        }
    }

    /// Reference.
    pub fn reference(&mut self) {
        self.referenced += 1;
    }

    /// Unreference.
    pub fn unreference(&mut self) {
        if self.is_referenced() {
            self.referenced -= 1;
        }
    }
}

/// Store operations.
pub enum Op<C: Codec, H: MultihashDigest> {
    /// Insert a block.
    Insert(Block<C, H>),
    /// Pin a block.
    Pin(Cid),
    /// Unpin a block.
    Unpin(Cid),
}

/// An atomic store transaction.
pub struct Transaction<C: Codec, H: MultihashDigest> {
    ops: Vec<Op<C, H>>,
}

impl<C: Codec, H: MultihashDigest> Transaction<C, H> {
    /// Creates a new transaction.
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Creates a transaction with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ops: Vec::with_capacity(capacity),
        }
    }

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Len.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Increases the pin count of a block.
    pub fn pin(&mut self, cid: Cid) {
        self.ops.push(Op::Pin(cid));
    }

    /// Decreases the pin count of a block.
    pub fn unpin(&mut self, cid: Cid) {
        self.ops.push(Op::Unpin(cid));
    }

    /// Update a block.
    ///
    /// Pins the new block and unpins the old one.
    pub fn update(&mut self, old: Option<Cid>, new: Cid) {
        self.pin(new);
        if let Some(old) = old {
            self.unpin(old);
        }
    }

    /// Inserts a block.
    pub fn insert(&mut self, block: Block<C, H>) {
        self.ops.push(Op::Insert(block));
    }

    /// Encodes a type into a block and inserts it.
    pub fn encode<CE: Codec, T: Encode<CE> + ?Sized>(
        &mut self,
        codec: CE,
        hash: u64,
        value: &T,
    ) -> Result<Cid>
    where
        CE: Into<C>,
    {
        let block = Block::<C, H>::encode(codec, hash, value)?;
        let cid = block.cid().clone();
        self.insert(block);
        Ok(cid)
    }
}

impl<C: Codec, H: MultihashDigest> IntoIterator for Transaction<C, H> {
    type Item = Op<C, H>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.into_iter()
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

    /// Returns a block from the store. If the store supports networking and the block is not
    /// in the store it fetches it from the network. Dropping the future cancels the request.
    /// This will not insert the block into the store.
    ///
    /// If the block wasn't found it returns a `BlockNotFound` error.
    async fn get(&self, cid: Cid) -> Result<Block<Self::Codec, Self::Multihash>>;

    /// Commits a transaction to the store.
    async fn commit(&self, tx: Transaction<Self::Codec, Self::Multihash>) -> Result<()>;

    /// Returns the status of a block.
    async fn status(&self, cid: &Cid) -> Result<Option<Status>>;

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
    async fn sync(&self, old: Option<Cid>, new: Cid) -> Result<()>
    where
        Ipld: Decode<Self::Codec>,
    {
        let mut visited = HashSet::new();
        let mut tx = Transaction::new();
        let mut stack = vec![new.clone()];
        while let Some(cid) = stack.pop() {
            if visited.contains(&cid) {
                continue;
            }
            visited.insert(cid.clone());
            if self.status(&cid).await?.is_none() {
                let block = self.get(cid).await?;
                for r in block.references()? {
                    stack.push(r);
                }
                tx.insert(block);
            }
        }
        tx.update(old, new);
        self.commit(tx).await
    }

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
