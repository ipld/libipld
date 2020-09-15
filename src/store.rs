//! Store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use async_trait::async_trait;
use std::borrow::Cow;
use std::collections::{HashSet, VecDeque};

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
    const MAX_BLOCK_SIZE: usize = usize::MAX;
    type Codecs = crate::Multicodec;
    type Hashes = crate::multihash::Multihash;
}

/// Store operations.
pub enum Op<'a, S> {
    /// Insert block.
    Insert(Block<S>, HashSet<Cid>),
    /// Pin a block.
    Pin(Cow<'a, Cid>),
    /// Unpin a block.
    Unpin(Cow<'a, Cid>),
}

/// An atomic store transaction.
pub struct Transaction<'cid, S> {
    ops: VecDeque<Op<'cid, S>>,
}

impl<'cid, S: StoreParams> Default for Transaction<'cid, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'cid, S: StoreParams> Transaction<'cid, S> {
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
    pub fn pin<'a: 'cid>(&mut self, cid: Cow<'a, Cid>) {
        self.ops.push_back(Op::Pin(cid));
    }

    /// Decreases the pin count of a block.
    pub fn unpin<'a: 'cid>(&mut self, cid: Cow<'a, Cid>) {
        self.ops.push_back(Op::Unpin(cid));
    }

    /// Update a block.
    ///
    /// Pins the new block and unpins the old one.
    pub fn update<'a: 'cid>(&mut self, old: Option<Cow<'a, Cid>>, new: Cow<'a, Cid>) {
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
    pub fn import(&mut self, block: Block<S>) -> Result<()>
    where
        Ipld: Decode<S::Codecs>,
    {
        let refs = block.references()?;
        self.ops.push_front(Op::Insert(block, refs));
        Ok(())
    }

    /// Inserts a block.
    ///
    /// This will add the block to the end of the transaction.
    pub fn insert(&mut self, block: Block<S>) -> Result<()>
    where
        Ipld: Decode<S::Codecs>,
    {
        let refs = block.references()?;
        self.ops.push_back(Op::Insert(block, refs));
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
        let block = Block::<S>::encode(codec, hash, value)?;
        let cid = block.cid().clone();
        self.insert(block)?;
        Ok(cid)
    }
}

impl<'a, 'cid, S: StoreParams> IntoIterator for &'a Transaction<'cid, S> {
    type Item = &'a Op<'cid, S>;
    type IntoIter = std::collections::vec_deque::Iter<'a, Op<'cid, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.ops.iter()
    }
}

impl<'cid, S: StoreParams> IntoIterator for Transaction<'cid, S> {
    type Item = Op<'cid, S>;
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
    async fn get(&self, cid: &Cid) -> Result<Block<Self::Params>>;

    /// Commits a transaction to the store.
    async fn commit(&self, tx: Transaction<'_, Self::Params>) -> Result<()>;

    /// Inserts a block into the store.
    async fn insert(&self, block: Block<Self::Params>) -> Result<()>
    where
        Ipld: Decode<<Self::Params as StoreParams>::Codecs>,
    {
        let mut tx = Transaction::with_capacity(1);
        tx.insert(block)?;
        self.commit(tx).await
    }

    /// Unpins a block from the store.
    async fn unpin<'a, I: Into<Cow<'a, Cid>> + Send>(&self, cid: I) -> Result<()> {
        let mut tx = Transaction::with_capacity(1);
        tx.unpin(cid.into());
        self.commit(tx).await
    }

    /// Resolves a path recursively and returns the ipld.
    async fn query(&self, path: &DagPath<'_>) -> Result<Ipld>
    where
        Ipld: Decode<<Self::Params as StoreParams>::Codecs>,
    {
        let mut root = self.get(path.root()).await?.ipld()?;
        let mut ipld = &root;
        for segment in path.path().iter() {
            ipld = ipld.get(segment)?;
            if let Ipld::Link(cid) = ipld {
                root = self.get(cid).await?.ipld()?;
                ipld = &root;
            }
        }
        Ok(ipld.clone())
    }

    /// Recursively gets a block and all it's references, inserts them into the store,
    /// pins the root and unpins the old root.
    ///
    /// If a block wasn't found it returns a `BlockNotFound` error without modifying the store.
    async fn sync<'a, 'old: 'a, 'new: 'a>(
        &'a self,
        old: Option<Cow<'new, Cid>>,
        new: Cow<'new, Cid>,
    ) -> Result<()>
    where
        Ipld: Decode<<Self::Params as StoreParams>::Codecs>,
    {
        let mut visited = HashSet::new();
        let mut tx = Transaction::new();
        let mut stack: Vec<Cid> = vec![(&*new).clone()];
        while let Some(cid) = stack.pop() {
            if visited.contains(&cid) {
                continue;
            }
            let block = self.get(&cid).await?;
            for r in block.references()? {
                stack.push(r);
            }
            tx.import(block)?;
            visited.insert(cid);
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
