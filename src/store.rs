//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::{BlockTooLarge, Result};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use async_trait::async_trait;
use std::borrow::Borrow;
use std::collections::{HashSet, VecDeque};

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
}

/// The store parameters.
pub trait StoreParams: Clone + Send + Sync {
    /// The multihash type of the store.
    type Hashes: MultihashDigest;
    /// The codec type of the store.
    type Codecs: Codec;
    /// The maximum block size supported by the store.
    const MAX_BLOCK_SIZE: usize;
}

/// Default store parameters.
#[derive(Clone)]
pub struct DefaultStoreParams;

impl StoreParams for DefaultStoreParams {
    const MAX_BLOCK_SIZE: usize = crate::MAX_BLOCK_SIZE;
    type Codecs = crate::Multicodec;
    type Hashes = crate::multihash::Multihash;
}

/// Type alias for a store compatible block.
pub type StoreBlock<S> = Block<<S as StoreParams>::Codecs, <S as StoreParams>::Hashes>;

/// Block info.
#[derive(Debug)]
pub struct BlockInfo<S: StoreParams> {
    block: Block<S::Codecs, S::Hashes>,
    refs: HashSet<Cid>,
    referrers: HashSet<Cid>,
    pinned: u32,
}

impl<S: StoreParams> core::hash::Hash for BlockInfo<S> {
    fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
        self.block.hash(hasher)
    }
}

impl<S: StoreParams> PartialEq for BlockInfo<S> {
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block
    }
}

impl<S: StoreParams> Eq for BlockInfo<S> {}

impl<S: StoreParams> Borrow<Cid> for BlockInfo<S> {
    fn borrow(&self) -> &Cid {
        self.block.borrow()
    }
}

impl<S: StoreParams> BlockInfo<S>
where
    Ipld: Decode<S::Codecs>,
{
    /// Creates a new `BlockInfo`.
    pub fn new(block: StoreBlock<S>) -> Result<Self> {
        if block.data().len() > S::MAX_BLOCK_SIZE {
            return Err(BlockTooLarge(block.data().len()).into());
        }
        let refs = block.ipld()?.references();
        Ok(Self {
            block,
            refs,
            referrers: Default::default(),
            pinned: 0,
        })
    }

    /// Block.
    pub fn block(&self) -> &StoreBlock<S> {
        &self.block
    }

    /// Refs.
    pub fn refs(&self) -> impl Iterator<Item = &Cid> {
        self.refs.iter()
    }

    /// Referrers.
    pub fn referrers(&self) -> impl Iterator<Item = &Cid> {
        self.referrers.iter()
    }

    /// Pin.
    pub fn pin(&mut self) {
        self.pinned += 1;
    }

    /// Unpin.
    pub fn unpin(&mut self) {
        self.pinned -= 1;
    }

    /// Add referrer.
    pub fn reference(&mut self, cid: Cid) {
        self.referrers.insert(cid);
    }

    /// Remove referrer.
    pub fn unreference(&mut self, cid: &Cid) {
        self.referrers.remove(cid);
    }

    /// Returns the status of a block.
    pub fn status(&self) -> Status {
        Status::new(self.pinned, self.referrers.len() as u32)
    }

    /// Remove returns the list of references.
    pub fn remove(self) -> HashSet<Cid> {
        self.refs
    }
}

/// Store operations.
pub enum Op<S: StoreParams> {
    /// Insert block.
    Insert(BlockInfo<S>),
    /// Pin a block.
    Pin(Cid),
    /// Unpin a block.
    Unpin(Cid),
}

/// An atomic store transaction.
pub struct Transaction<S: StoreParams> {
    ops: VecDeque<Op<S>>,
}

impl<S: StoreParams> Default for Transaction<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: StoreParams> Transaction<S> {
    /// Creates a new transaction.
    pub fn new() -> Self {
        Self {
            ops: VecDeque::new(),
        }
    }

    /// Creates a transaction with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ops: VecDeque::with_capacity(capacity),
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
        self.ops.push_back(Op::Pin(cid));
    }

    /// Decreases the pin count of a block.
    pub fn unpin(&mut self, cid: Cid) {
        self.ops.push_back(Op::Unpin(cid));
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

    /// Imports a block.
    ///
    /// This will add the block to the front of the transaction.
    ///
    /// When importing blocks from a network, they'll be fetched in
    /// reverse order. This ensures that when inserting the block
    /// all it's references have been inserted.
    pub fn import(&mut self, block: StoreBlock<S>) -> Result<()>
    where
        Ipld: Decode<S::Codecs>,
    {
        let info = BlockInfo::new(block)?;
        self.ops.push_front(Op::Insert(info));
        Ok(())
    }

    /// Inserts a block.
    ///
    /// This will add the block to the end of the transaction.
    pub fn insert(&mut self, block: StoreBlock<S>) -> Result<()>
    where
        Ipld: Decode<S::Codecs>,
    {
        let info = BlockInfo::new(block)?;
        self.ops.push_back(Op::Insert(info));
        Ok(())
    }

    /// Creates a block.
    ///
    /// This will add the block to the end of the transaction.
    ///
    /// When constructing blocks, the all references need to be
    /// created before the referrer.
    pub fn create<CE: Codec, T: Encode<CE> + ?Sized>(
        &mut self,
        codec: CE,
        hash: u64,
        value: &T,
    ) -> Result<Cid>
    where
        CE: Into<S::Codecs>,
        Ipld: Decode<S::Codecs>,
    {
        let block = StoreBlock::<S>::encode(codec, hash, value)?;
        let cid = block.cid().clone();
        self.insert(block)?;
        Ok(cid)
    }
}

impl<'a, S: StoreParams> IntoIterator for &'a Transaction<S> {
    type Item = &'a Op<S>;
    type IntoIter = std::collections::vec_deque::Iter<'a, Op<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.iter()
    }
}

impl<S: StoreParams> IntoIterator for Transaction<S> {
    type Item = Op<S>;
    type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.into_iter()
    }
}

/// Implementable by ipld stores.
#[async_trait]
pub trait Store: Clone + Send + Sync {
    /// Store parameters.
    type Params: StoreParams;

    /// Returns a block from the store. If the store supports networking and the block is not
    /// in the store it fetches it from the network. Dropping the future cancels the request.
    /// This will not insert the block into the store.
    ///
    /// If the block wasn't found it returns a `BlockNotFound` error.
    async fn get(&self, cid: Cid) -> Result<StoreBlock<Self::Params>>;

    /// Commits a transaction to the store.
    async fn commit(&self, tx: Transaction<Self::Params>) -> Result<()>;

    /// Returns the status of a block.
    async fn status(&self, cid: &Cid) -> Result<Option<Status>>;

    /// Unpins a block from the store.
    async fn unpin(&self, cid: &Cid) -> Result<()> {
        let mut tx = Transaction::with_capacity(1);
        tx.unpin(cid.clone());
        self.commit(tx).await
    }

    /// Resolves a path recursively and returns the ipld.
    async fn query(&self, path: &DagPath<'_>) -> Result<Ipld>
    where
        Ipld: Decode<<Self::Params as StoreParams>::Codecs>,
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

    /// Recursively gets a block and all it's references, inserts them into the store,
    /// pins the root and unpins the old root.
    ///
    /// If a block wasn't found it returns a `BlockNotFound` error without modifying the store.
    async fn sync(&self, old: Option<Cid>, new: Cid) -> Result<()>
    where
        Ipld: Decode<<Self::Params as StoreParams>::Codecs>,
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
                tx.import(block)?;
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
