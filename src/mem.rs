//! Reference implementation of the store traits.
use crate::cid::Cid;
use crate::codec::Decode;
use crate::error::{BlockNotFound, Result};
use crate::ipld::Ipld;
use crate::store::{
    AliasStore, BlockInfo, Op, Status, Store, StoreBlock, StoreParams, Transaction,
};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

/// Models a network for testing.
pub struct GlobalStore<S: StoreParams> {
    blocks: RwLock<HashSet<StoreBlock<S>>>,
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
    async fn get(&self, cid: &Cid) -> Result<StoreBlock<S>> {
        if let Some(block) = self.blocks.read().await.get(cid) {
            Ok(block.clone())
        } else {
            Err(BlockNotFound(cid.to_string()).into())
        }
    }

    async fn insert(&self, block: StoreBlock<S>) {
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

    async fn get(&self, cid: &Cid) -> Result<StoreBlock<S>> {
        if let Some(info) = self.blocks.get(cid) {
            Ok(info.block().clone())
        } else {
            self.global.get(cid).await
        }
    }

    fn status(&self, cid: &Cid) -> Option<Status> {
        self.blocks.get(cid).map(|info| info.status())
    }

    fn blocks(&self) -> Vec<Cid> {
        self.blocks
            .iter()
            .map(|info| info.block().cid().clone())
            .collect()
    }

    fn verify_transaction(&self, tx: &Transaction<S>) -> Result<()> {
        let mut inserts = HashSet::with_capacity(tx.len());
        for op in tx {
            match op {
                Op::Insert(info) => {
                    if self.status(info.block().cid()).is_some() {
                        continue;
                    }
                    for cid in info.refs() {
                        if !inserts.contains(cid) && self.status(cid).is_none() {
                            return Err(BlockNotFound(cid.to_string()).into());
                        }
                    }
                    inserts.insert(info.block().cid());
                }
                Op::Pin(cid) => {
                    if !inserts.contains(&cid) && self.status(&cid).is_none() {
                        return Err(BlockNotFound(cid.to_string()).into());
                    }
                }
                Op::Unpin(cid) => {
                    if !inserts.contains(&cid) && self.status(&cid).is_none() {
                        return Err(BlockNotFound(cid.to_string()).into());
                    }
                }
            }
        }
        Ok(())
    }

    async fn commit(&mut self, tx: Transaction<S>) -> Result<()> {
        let mut dead = Vec::with_capacity(tx.len());
        self.verify_transaction(&tx)?;
        for op in tx {
            match op {
                Op::Insert(info) => {
                    for cid in info.refs() {
                        let mut info2 = self.blocks.take(cid).unwrap();
                        info2.reference(info.block().cid().clone());
                        self.blocks.insert(info2);
                    }
                    self.global.insert(info.block().clone()).await;
                    self.blocks.insert(info);
                }
                Op::Pin(cid) => {
                    let mut info = self.blocks.take(&cid).unwrap();
                    info.pin();
                    self.blocks.insert(info);
                }
                Op::Unpin(cid) => {
                    let mut info = self.blocks.take(&cid).unwrap();
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
}

#[async_trait]
impl<S: StoreParams> Store for MemStore<S>
where
    Ipld: Decode<S::Codecs>,
{
    type Params = S;

    async fn get(&self, cid: Cid) -> Result<StoreBlock<S>> {
        self.local.read().await.get(&cid).await
    }

    async fn commit(&self, tx: Transaction<S>) -> Result<()> {
        self.local.write().await.commit(tx).await
    }

    async fn status(&self, cid: &Cid) -> Result<Option<Status>> {
        Ok(self.local.read().await.status(cid))
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
    use crate::codec_impl::Multicodec;
    use crate::ipld;
    use crate::ipld::Ipld;
    use crate::multihash::{Multihash, SHA2_256};
    use crate::store::{DefaultStoreParams, Store};

    fn create_block(ipld: &Ipld) -> Block<Multicodec, Multihash> {
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
        tx.pin(b.cid().clone());
        tx.pin(c.cid().clone());
        store.commit(tx).await.unwrap();
        assert_eq!(store.status(a.cid()).await?, Some(Status::new(0, 2)));
        assert_eq!(store.status(b.cid()).await?, Some(Status::new(1, 0)));
        assert_eq!(store.status(c.cid()).await?, Some(Status::new(1, 0)));

        store.unpin(b.cid()).await?;
        assert_eq!(store.status(a.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(store.status(b.cid()).await?, None);
        assert_eq!(store.status(c.cid()).await?, Some(Status::new(1, 0)));

        store.unpin(c.cid()).await?;
        assert_eq!(store.status(a.cid()).await?, None);
        assert_eq!(store.status(b.cid()).await?, None);
        assert_eq!(store.status(c.cid()).await?, None);

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
        tx.pin(b.cid().clone());
        tx.pin(c.cid().clone());
        store.commit(tx).await?;
        assert_eq!(store.status(a.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(store.status(b.cid()).await?, Some(Status::new(2, 0)));
        assert_eq!(store.status(c.cid()).await?, Some(Status::new(2, 0)));

        store.unpin(b.cid()).await?;
        assert_eq!(store.status(a.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(store.status(b.cid()).await?, Some(Status::new(1, 0)));
        assert_eq!(store.status(c.cid()).await?, Some(Status::new(1, 0)));

        store.unpin(c.cid()).await?;
        assert_eq!(store.status(a.cid()).await?, None);
        assert_eq!(store.status(b.cid()).await?, None);
        assert_eq!(store.status(c.cid()).await?, None);

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
        tx.pin(c1.cid().clone());
        local1.commit(tx).await?;
        assert_eq!(local1.status(a1.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local1.status(b1.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local1.status(c1.cid()).await?, Some(Status::new(1, 0)));

        local2.sync(None, c1.cid().clone()).await?;
        assert_eq!(local2.status(a1.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local2.status(b1.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local2.status(c1.cid()).await?, Some(Status::new(1, 0)));

        let mut tx = Transaction::with_capacity(4);
        tx.insert(b2.clone())?;
        tx.insert(c2.clone())?;
        tx.pin(c2.cid().clone());
        tx.unpin(c1.cid().clone());
        local2.commit(tx).await?;
        assert_eq!(local2.status(a1.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local2.status(b1.cid()).await?, None);
        assert_eq!(local2.status(c1.cid()).await?, None);
        assert_eq!(local2.status(b2.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local2.status(c2.cid()).await?, Some(Status::new(1, 0)));

        local1
            .sync(Some(c1.cid().clone()), c2.cid().clone())
            .await?;
        assert_eq!(local1.status(a1.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local1.status(b1.cid()).await?, None);
        assert_eq!(local1.status(c1.cid()).await?, None);
        assert_eq!(local1.status(b2.cid()).await?, Some(Status::new(0, 1)));
        assert_eq!(local1.status(c2.cid()).await?, Some(Status::new(1, 0)));

        let mut tx = Transaction::with_capacity(1);
        tx.unpin(c2.cid().clone());
        local1.commit(tx).await?;
        assert_eq!(local1.status(a1.cid()).await?, None);
        assert_eq!(local1.status(b2.cid()).await?, None);
        assert_eq!(local1.status(c2.cid()).await?, None);

        let mut tx = Transaction::with_capacity(1);
        tx.unpin(c2.cid().clone());
        local2.commit(tx).await?;
        assert_eq!(local2.status(a1.cid()).await?, None);
        assert_eq!(local2.status(b2.cid()).await?, None);
        assert_eq!(local2.status(c2.cid()).await?, None);

        Ok(())
    }
}
