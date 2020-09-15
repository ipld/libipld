//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::Decode;
use crate::error::{BlockNotFound, Result};
use crate::ipld::Ipld;
use crate::store::{AliasStore, Op, Store, StoreParams, Transaction};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

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

/// Block info.
#[derive(Debug)]
pub struct BlockInfo<S: StoreParams> {
    block: Block<S>,
    refs: HashSet<Cid>,
    referrers: HashSet<Cid>,
    pinned: u32,
}

impl<S: StoreParams> Clone for BlockInfo<S> {
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            refs: self.refs.clone(),
            referrers: self.referrers.clone(),
            pinned: self.pinned,
        }
    }
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

impl<S: StoreParams> BlockInfo<S> {
    /// Creates a new `BlockInfo`.
    pub fn new(block: Block<S>, refs: HashSet<Cid>) -> Self {
        Self {
            block,
            refs,
            referrers: Default::default(),
            pinned: 0,
        }
    }

    /// Block.
    pub fn block(&self) -> &Block<S> {
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

/// Models a network for testing.
pub struct GlobalStore<S: StoreParams> {
    blocks: RwLock<HashSet<Block<S>>>,
    aliases: RwLock<HashMap<Vec<u8>, Cid>>,
}

impl<S: StoreParams> Default for GlobalStore<S> {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            aliases: Default::default(),
        }
    }
}

impl<S: StoreParams> GlobalStore<S> {
    async fn get(&self, cid: &Cid) -> Result<Block<S>> {
        if let Some(block) = self.blocks.read().await.get(cid) {
            Ok(block.clone())
        } else {
            Err(BlockNotFound(cid.to_string()).into())
        }
    }

    async fn insert(&self, block: Block<S>) {
        self.blocks.write().await.insert(block);
    }

    async fn alias(&self, alias: &[u8], cid: &Cid) {
        self.aliases
            .write()
            .await
            .insert(alias.to_vec(), cid.clone());
    }

    async fn unalias(&self, alias: &[u8]) {
        self.aliases.write().await.remove(alias);
    }

    async fn resolve<'a>(&'a self, alias: &'a [u8]) -> Option<Cid> {
        self.aliases.read().await.get(alias).cloned()
    }
}

struct LocalStore<S: StoreParams> {
    global: Arc<GlobalStore<S>>,
    blocks: HashSet<BlockInfo<S>>,
}

impl<S: StoreParams> Default for LocalStore<S> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            blocks: Default::default(),
        }
    }
}

impl<S: StoreParams> LocalStore<S>
where
    Ipld: Decode<S::Codecs>,
{
    fn new(global: Arc<GlobalStore<S>>) -> Self {
        Self {
            global,
            ..Default::default()
        }
    }

    async fn get(&self, cid: &Cid) -> Result<Block<S>> {
        if let Some(info) = self.blocks.get(cid) {
            Ok(info.block().clone())
        } else {
            self.global.get(cid).await
        }
    }

    fn blocks(&self) -> Vec<Cid> {
        self.blocks
            .iter()
            .map(|info| info.block().cid().clone())
            .collect()
    }

    fn info<'a>(&'a self, cid: &'a Cid) -> Option<&'a BlockInfo<S>> {
        self.blocks.get(cid)
    }

    fn status(&self, cid: &Cid) -> Option<Status> {
        self.info(cid).map(|info| info.status())
    }

    fn verify_transaction(&self, tx: &Transaction<S>) -> Result<()> {
        let mut inserts = HashSet::with_capacity(tx.len());
        for op in tx {
            match op {
                Op::Insert(block, refs) => {
                    if self.status(block.cid()).is_some() {
                        continue;
                    }
                    for cid in refs {
                        if !inserts.contains(cid) && self.status(cid).is_none() {
                            return Err(BlockNotFound(cid.to_string()).into());
                        }
                    }
                    inserts.insert(block.cid());
                }
                Op::Pin(cid) => {
                    if !inserts.contains(cid.deref()) && self.status(cid).is_none() {
                        return Err(BlockNotFound(cid.to_string()).into());
                    }
                }
                Op::Unpin(cid) => {
                    if !inserts.contains(cid.deref()) && self.status(cid).is_none() {
                        return Err(BlockNotFound(cid.to_string()).into());
                    }
                }
            }
        }
        Ok(())
    }

    async fn commit(&mut self, tx: Transaction<'_, S>) -> Result<()> {
        let mut dead = Vec::with_capacity(tx.len());
        self.verify_transaction(&tx)?;
        for op in tx {
            match op {
                Op::Insert(block, refs) => {
                    for cid in &refs {
                        let mut info2 = self.blocks.take(cid).unwrap();
                        info2.reference(block.cid().clone());
                        self.blocks.insert(info2);
                    }
                    self.global.insert(block.clone()).await;
                    self.blocks.insert(BlockInfo::new(block, refs));
                }
                Op::Pin(cid) => {
                    let mut info = self.blocks.take(cid.deref()).unwrap();
                    info.pin();
                    self.blocks.insert(info);
                }
                Op::Unpin(cid) => {
                    let mut info = self.blocks.take(cid.deref()).unwrap();
                    info.unpin();
                    if info.status().is_dead() {
                        dead.push(cid);
                    }
                    self.blocks.insert(info);
                }
            }
        }
        for cid in dead {
            self.remove(&cid);
        }
        Ok(())
    }

    fn remove_block(&mut self, cid: &Cid) -> HashSet<Cid> {
        let info = self.blocks.take(cid).unwrap();
        for cid in info.refs() {
            if let Some(mut info2) = self.blocks.take(cid) {
                info2.unreference(info.block().cid());
                self.blocks.insert(info2);
            }
        }
        info.remove()
    }

    fn remove(&mut self, cid: &Cid) {
        if let Some(status) = self.status(cid) {
            if status.is_dead() {
                for cid in self.remove_block(cid) {
                    self.remove(&cid);
                }
            }
        }
    }
}

/// A memory backed store
#[derive(Clone)]
pub struct MemStore<S: StoreParams> {
    global: Arc<GlobalStore<S>>,
    local: Arc<RwLock<LocalStore<S>>>,
}

impl<S: StoreParams> Default for MemStore<S> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            local: Default::default(),
        }
    }
}

impl<S: StoreParams> MemStore<S>
where
    Ipld: Decode<S::Codecs>,
{
    /// Create a new empty `MemStore`
    pub fn new(global: Arc<GlobalStore<S>>) -> Self {
        Self {
            global: global.clone(),
            local: Arc::new(RwLock::new(LocalStore::new(global))),
        }
    }

    /// Returns a vec of all cid's in the store.
    pub async fn blocks(&self) -> Vec<Cid> {
        self.local.read().await.blocks()
    }

    /// Returns the status of a block.
    pub async fn status(&self, cid: &Cid) -> Option<Status> {
        self.local.read().await.status(cid)
    }

    /// Returns the block info.
    pub async fn info<'a>(&'a self, cid: &'a Cid) -> Option<BlockInfo<S>> {
        self.local.read().await.info(cid).cloned()
    }
}

#[async_trait]
impl<S: StoreParams> Store for MemStore<S>
where
    Ipld: Decode<S::Codecs>,
{
    type Params = S;

    async fn get(&self, cid: &Cid) -> Result<Block<S>> {
        self.local.read().await.get(cid).await
    }

    async fn commit(&self, tx: Transaction<'_, S>) -> Result<()> {
        self.local.write().await.commit(tx).await
    }
}

#[async_trait]
impl<S: StoreParams> AliasStore for MemStore<S>
where
    Ipld: Decode<S::Codecs>,
{
    async fn alias(&self, alias: &[u8], cid: &Cid) -> Result<()> {
        self.global.alias(alias, cid).await;
        Ok(())
    }

    async fn unalias(&self, alias: &[u8]) -> Result<()> {
        self.global.unalias(alias).await;
        Ok(())
    }

    async fn resolve(&self, alias: &[u8]) -> Result<Option<Cid>> {
        Ok(self.global.resolve(alias).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;
    use crate::cbor::DagCborCodec;
    use crate::ipld;
    use crate::ipld::Ipld;
    use crate::multihash::SHA2_256;
    use crate::store::{DefaultStoreParams, Store};

    fn create_block(ipld: &Ipld) -> Block<DefaultStoreParams> {
        Block::encode(DagCborCodec, SHA2_256, ipld).unwrap()
    }

    #[async_std::test]
    async fn test_gc() -> Result<()> {
        let store = MemStore::<DefaultStoreParams>::default();
        let a = create_block(&ipld!({ "a": [] }));
        let b = create_block(&ipld!({ "b": [a.cid()] }));
        let c = create_block(&ipld!({ "c": [a.cid()] }));

        let mut tx = Transaction::with_capacity(5);
        tx.insert(a.clone())?;
        tx.insert(b.clone())?;
        tx.insert(c.clone())?;
        tx.pin(b.cid());
        tx.pin(c.cid());
        store.commit(tx).await.unwrap();
        assert_eq!(store.status(a.cid()).await, Some(Status::new(0, 2)));
        assert_eq!(store.status(b.cid()).await, Some(Status::new(1, 0)));
        assert_eq!(store.status(c.cid()).await, Some(Status::new(1, 0)));

        store.unpin(b.cid()).await?;
        assert_eq!(store.status(a.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(store.status(b.cid()).await, None);
        assert_eq!(store.status(c.cid()).await, Some(Status::new(1, 0)));

        store.unpin(c.cid()).await?;
        assert_eq!(store.status(a.cid()).await, None);
        assert_eq!(store.status(b.cid()).await, None);
        assert_eq!(store.status(c.cid()).await, None);

        Ok(())
    }

    #[async_std::test]
    async fn test_gc_2() -> Result<()> {
        let store = MemStore::<DefaultStoreParams>::default();
        let a = create_block(&ipld!({ "a": [] }));
        let b = create_block(&ipld!({ "b": [a.cid()] }));
        let c = b.clone();

        let mut tx = Transaction::with_capacity(5);
        tx.insert(a.clone())?;
        tx.insert(b.clone())?;
        tx.insert(c.clone())?;
        tx.pin(b.cid());
        tx.pin(c.cid());
        store.commit(tx).await?;
        assert_eq!(store.status(a.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(store.status(b.cid()).await, Some(Status::new(2, 0)));
        assert_eq!(store.status(c.cid()).await, Some(Status::new(2, 0)));

        store.unpin(b.cid()).await?;
        assert_eq!(store.status(a.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(store.status(b.cid()).await, Some(Status::new(1, 0)));
        assert_eq!(store.status(c.cid()).await, Some(Status::new(1, 0)));

        store.unpin(c.cid()).await?;
        assert_eq!(store.status(a.cid()).await, None);
        assert_eq!(store.status(b.cid()).await, None);
        assert_eq!(store.status(c.cid()).await, None);

        Ok(())
    }

    #[async_std::test]
    async fn test_sync() -> Result<()> {
        let global = Arc::new(GlobalStore::<DefaultStoreParams>::default());
        let local1 = MemStore::new(global.clone());
        let local2 = MemStore::new(global.clone());
        let a1 = create_block(&ipld!({ "a": 0 }));
        let b1 = create_block(&ipld!({ "b": 0 }));
        let c1 = create_block(&ipld!({ "c": [a1.cid(), b1.cid()] }));
        let b2 = create_block(&ipld!({ "b": 1 }));
        let c2 = create_block(&ipld!({ "c": [a1.cid(), b2.cid()] }));

        let mut tx = Transaction::with_capacity(4);
        tx.insert(a1.clone())?;
        tx.insert(b1.clone())?;
        tx.insert(c1.clone())?;
        tx.pin(c1.cid());
        local1.commit(tx).await?;
        assert_eq!(local1.status(a1.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local1.status(b1.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local1.status(c1.cid()).await, Some(Status::new(1, 0)));

        local2.sync(None::<Cid>, c1.cid()).await?;
        assert_eq!(local2.status(a1.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local2.status(b1.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local2.status(c1.cid()).await, Some(Status::new(1, 0)));

        let mut tx = Transaction::with_capacity(4);
        tx.insert(b2.clone())?;
        tx.insert(c2.clone())?;
        tx.pin(c2.cid());
        tx.unpin(c1.cid());
        local2.commit(tx).await?;
        assert_eq!(local2.status(a1.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local2.status(b1.cid()).await, None);
        assert_eq!(local2.status(c1.cid()).await, None);
        assert_eq!(local2.status(b2.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local2.status(c2.cid()).await, Some(Status::new(1, 0)));

        local1.sync(Some(c1.cid()), c2.cid()).await?;
        assert_eq!(local1.status(a1.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local1.status(b1.cid()).await, None);
        assert_eq!(local1.status(c1.cid()).await, None);
        assert_eq!(local1.status(b2.cid()).await, Some(Status::new(0, 1)));
        assert_eq!(local1.status(c2.cid()).await, Some(Status::new(1, 0)));

        let mut tx = Transaction::with_capacity(1);
        tx.unpin(c2.cid());
        local1.commit(tx).await?;
        assert_eq!(local1.status(a1.cid()).await, None);
        assert_eq!(local1.status(b2.cid()).await, None);
        assert_eq!(local1.status(c2.cid()).await, None);

        let mut tx = Transaction::with_capacity(1);
        tx.unpin(c2.cid());
        local2.commit(tx).await?;
        assert_eq!(local2.status(a1.cid()).await, None);
        assert_eq!(local2.status(b2.cid()).await, None);
        assert_eq!(local2.status(c2.cid()).await, None);

        Ok(())
    }
}
