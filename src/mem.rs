//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode};
use crate::error::{BlockNotFound, BlockTooLarge, EmptyBatch, Result};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::store::{AliasStore, Status, Store, StoreResult};
use crate::MAX_BLOCK_SIZE;
use async_std::sync::{Arc, RwLock};
use core::marker::PhantomData;
use std::collections::{HashMap, HashSet};

struct BlockInfo {
    data: Box<[u8]>,
    refs: HashSet<Cid>,
    referenced: usize,
    pinned: usize,
}

/// Models a network for testing.
pub struct GlobalStore<C, M> {
    _marker: PhantomData<(C, M)>,
    blocks: RwLock<HashMap<Cid, Box<[u8]>>>,
    aliases: RwLock<HashMap<Box<[u8]>, Cid>>,
}

impl<C, M> Default for GlobalStore<C, M> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
            blocks: Default::default(),
            aliases: Default::default(),
        }
    }
}

impl<C: Codec, M: MultihashDigest> GlobalStore<C, M> {
    async fn get(&self, cid: Cid) -> Result<Block<C, M>> {
        if let Some(data) = self.blocks.read().await.get(&cid).cloned() {
            Ok(Block::new(cid, data))
        } else {
            Err(BlockNotFound(cid.to_string()).into())
        }
    }

    async fn insert(&self, block: &Block<C, M>) {
        self.blocks.write().await.insert(block.cid.clone(), block.data.clone());
    }

    async fn alias(&self, alias: &[u8], block: &Block<C, M>) {
        self.aliases
            .write()
            .await
            .insert(alias.to_vec().into_boxed_slice(), block.cid.clone());
    }

    async fn unalias(&self, alias: &[u8]) {
        self.aliases.write().await.remove(alias);
    }

    async fn resolve<'a>(&'a self, alias: &'a [u8]) -> Option<Cid> {
        self.aliases.read().await.get(alias).cloned()
    }
}

struct LocalStore<C, M> {
    global: Arc<GlobalStore<C, M>>,
    blocks: HashMap<Cid, BlockInfo>,
}

impl<C, M> Default for LocalStore<C, M> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            blocks: Default::default(),
        }
    }
}

impl<C: Codec, M: MultihashDigest> LocalStore<C, M>
where
    Ipld: Decode<C>,
{
    /// Create a new empty `InnerStore`
    fn new(global: Arc<GlobalStore<C, M>>) -> Self {
        Self {
            global,
            ..Default::default()
        }
    }

    async fn pin(&mut self, cid: &Cid) -> Result<()> {
        if let Some(info) = self.blocks.get_mut(cid) {
            info.pinned += 1;
        } else {
            return Err(BlockNotFound(cid.to_string()).into());
        }
        Ok(())
    }

    async fn unpin(&mut self, cid: &Cid) -> Result<()> {
        if let Some(info) = self.blocks.get_mut(cid) {
            if info.pinned > 0 {
                info.pinned -= 1;
            }
            if info.pinned == 0 {
                self.remove(cid);
            }
        } else {
            return Err(BlockNotFound(cid.to_string()).into());
        }
        Ok(())
    }

    async fn get(&self, cid: Cid) -> Result<Block<C, M>> {
        if let Some(info) = self.blocks.get(&cid) {
            Ok(Block::new(cid, info.data.clone()))
        } else {
            self.global.get(cid).await
        }
    }

    async fn sync(&mut self, cid: Cid) -> Result<Block<C, M>> {
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

    async fn status(&self, cid: &Cid) -> Status {
        if let Some(info) = self.blocks.get(&cid) {
            Status::new(info.pinned, info.referenced)
        } else {
            Status::new(0, 0)
        }
    }

    async fn insert_block(&mut self, block: &Block<C, M>) -> Result<()> {
        if self.blocks.contains_key(&block.cid) {
            return Ok(());
        }
        if block.data.len() > MAX_BLOCK_SIZE {
            return Err(BlockTooLarge(block.data.len()).into());
        }
        let refs = block.references()?;
        for cid in &refs {
            if !self.blocks.contains_key(cid) {
                return Err(BlockNotFound(cid.to_string()).into());
            }
        }
        for cid in &refs {
            let mut info = self.blocks.get_mut(cid).unwrap();
            info.referenced += 1;
        }
        let info = BlockInfo {
            data: block.data.clone(),
            refs,
            referenced: 0,
            pinned: 0,
        };
        self.blocks.insert(block.cid.clone(), info);
        self.global.insert(block).await;
        Ok(())
    }

    async fn insert(&mut self, block: &Block<C, M>) -> Result<()> {
        self.insert_block(&block).await?;
        self.pin(&block.cid).await?;
        Ok(())
    }

    async fn insert_batch(&mut self, batch: &[Block<C, M>]) -> Result<Cid> {
        let root = batch.last().ok_or(EmptyBatch)?.cid.clone();
        for block in batch {
            self.insert_block(block).await?;
        }
        self.pin(&root).await?;
        Ok(root)
    }

    fn remove_block(&mut self, cid: &Cid) -> HashSet<Cid> {
        let info = self.blocks.remove(cid).unwrap();
        for cid in &info.refs {
            if let Some(info) = self.blocks.get_mut(&cid) {
                info.referenced -= 1;
            }
        }
        info.refs
    }

    fn remove(&mut self, cid: &Cid) {
        if let Some(info) = self.blocks.get(cid) {
            if info.pinned < 1 && info.referenced < 1 {
                let refs = self.remove_block(cid);
                for cid in refs {
                    self.remove(&cid);
                }
            }
        }
    }

    fn blocks(&self) -> Vec<Cid> {
        self.blocks.iter().map(|(k, _)| k.clone()).collect()
    }
}

/// A memory backed store
#[derive(Clone)]
pub struct MemStore<C, M> {
    global: Arc<GlobalStore<C, M>>,
    local: Arc<RwLock<LocalStore<C, M>>>,
}

impl<C, M> Default for MemStore<C, M> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            local: Default::default(),
        }
    }
}

impl<C: Codec, M: MultihashDigest> MemStore<C, M>
where
    Ipld: Decode<C>,
{
    /// Create a new empty `MemStore`
    pub fn new(global: Arc<GlobalStore<C, M>>) -> Self {
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

impl<C: Codec, M: MultihashDigest> Store for MemStore<C, M>
where
    Ipld: Decode<C>,
{
    type Codec = C;
    type Multihash = M;
    const MAX_BLOCK_SIZE: usize = crate::MAX_BLOCK_SIZE;

    fn pin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move { self.local.write().await.pin(cid).await })
    }

    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move { self.local.write().await.unpin(cid).await })
    }

    fn get<'a>(&'a self, cid: Cid) -> StoreResult<'a, Block<C, M>> {
        Box::pin(async move { self.local.read().await.get(cid).await })
    }

    fn sync<'a>(&'a self, cid: Cid) -> StoreResult<'a, Block<C, M>> {
        Box::pin(async move { self.local.write().await.sync(cid).await })
    }

    fn status<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Status> {
        Box::pin(async move { Ok(self.local.read().await.status(cid).await) })
    }

    fn insert<'a>(&'a self, block: &'a Block<C, M>) -> StoreResult<'a, ()> {
        Box::pin(async move { self.local.write().await.insert(block).await })
    }

    fn insert_batch<'a>(&'a self, batch: &'a [Block<C, M>]) -> StoreResult<'a, Cid> {
        Box::pin(async move { self.local.write().await.insert_batch(batch).await })
    }

    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }
}

impl<C: Codec, M: MultihashDigest> AliasStore for MemStore<C, M>
where
    Ipld: Decode<C>,
{
    fn alias<'a>(&'a self, alias: &'a [u8], block: &'a Block<C, M>) -> StoreResult<'a, ()> {
        Box::pin(async move { Ok(self.global.alias(alias, block).await) })
    }

    fn unalias<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, ()> {
        Box::pin(async move { Ok(self.global.unalias(alias).await) })
    }

    fn resolve<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, Option<Cid>> {
        Box::pin(async move { Ok(self.global.resolve(alias).await) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;
    use crate::cbor::DagCborCodec;
    use crate::ipld;
    use crate::ipld::Ipld;
    use crate::multihash::{Multihash, SHA2_256};
    use crate::store::Store;

    async fn get_local<S: Store>(store: &S, cid: &Cid) -> Option<Ipld>
    where
        Ipld: Decode<S::Codec>,
    {
        if store.status(cid).await.unwrap().is_live() {
            get(store, cid).await
        } else {
            None
        }
    }

    async fn get<S: Store>(store: &S, cid: &Cid) -> Option<Ipld>
    where
        Ipld: Decode<S::Codec>,
    {
        let block = match store.get(cid.clone()).await {
            Ok(block) => block,
            Err(e) if e.downcast_ref::<BlockNotFound>().is_some() => return None,
            Err(e) => Err(e).unwrap(),
        };
        let ipld = block.decode::<_, Ipld>().unwrap();
        Some(ipld)
    }

    async fn insert<S: Store>(store: &S, ipld: &Ipld) -> Cid
    where
        S::Codec: From<DagCborCodec>,
    {
        let block = Block::encode(DagCborCodec, SHA2_256, ipld).unwrap();
        store.insert(&block).await.unwrap();
        block.cid
    }

    #[async_std::test]
    async fn test_gc() {
        let store = MemStore::<DagCborCodec, Multihash>::default();
        let a = insert(&store, &ipld!({ "a": [] })).await;
        let b = insert(&store, &ipld!({ "b": [&a] })).await;
        store.unpin(&a).await.unwrap();
        let c = insert(&store, &ipld!({ "c": [&a] })).await;
        assert!(get_local(&store, &a).await.is_some());
        assert!(get_local(&store, &b).await.is_some());
        assert!(get_local(&store, &c).await.is_some());
        store.unpin(&b).await.unwrap();
        assert!(get_local(&store, &a).await.is_some());
        assert!(get_local(&store, &b).await.is_none());
        assert!(get_local(&store, &c).await.is_some());
        store.unpin(&c).await.unwrap();
        assert!(get_local(&store, &a).await.is_none());
        assert!(get_local(&store, &b).await.is_none());
        assert!(get_local(&store, &c).await.is_none());
    }

    #[async_std::test]
    async fn test_gc_2() {
        let store = MemStore::<DagCborCodec, Multihash>::default();
        let a = insert(&store, &ipld!({ "a": [] })).await;
        let b = insert(&store, &ipld!({ "b": [&a] })).await;
        store.unpin(&a).await.unwrap();
        let c = insert(&store, &ipld!({ "b": [&a] })).await;
        assert!(get_local(&store, &a).await.is_some());
        assert!(get_local(&store, &b).await.is_some());
        assert!(get_local(&store, &c).await.is_some());
        store.unpin(&b).await.unwrap();
        assert!(get_local(&store, &a).await.is_some());
        assert!(get_local(&store, &b).await.is_some());
        assert!(get_local(&store, &c).await.is_some());
        store.unpin(&c).await.unwrap();
        assert!(get_local(&store, &a).await.is_none());
        assert!(get_local(&store, &b).await.is_none());
        assert!(get_local(&store, &c).await.is_none());
    }

    #[async_std::test]
    async fn test_sync() {
        let global = Arc::new(GlobalStore::<DagCborCodec, Multihash>::default());
        let local1 = MemStore::new(global.clone());
        let local2 = MemStore::new(global.clone());
        let a = insert(&local1, &ipld!({ "a": [] })).await;
        let b = insert(&local1, &ipld!({ "b": [&a] })).await;
        local2.sync(b.clone()).await.unwrap();
        assert!(get_local(&local2, &a).await.is_some());
        assert!(get_local(&local2, &b).await.is_some());
    }
}
