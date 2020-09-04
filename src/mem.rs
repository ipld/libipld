//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode};
use crate::error::{BlockNotFound, BlockTooLarge, EmptyBatch, Result};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::store::{AliasStore, Status, Store};
use crate::MAX_BLOCK_SIZE;
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
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
        self.blocks
            .write()
            .await
            .insert(block.cid.clone(), block.data.clone());
    }

    async fn alias(&self, alias: &[u8], cid: &Cid) {
        self.aliases
            .write()
            .await
            .insert(alias.to_vec().into_boxed_slice(), cid.clone());
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

    async fn status(&self, cid: &Cid) -> Option<Status> {
        self.blocks
            .get(&cid)
            .map(|info| Status::new(info.pinned, info.referenced))
    }

    async fn _insert(&mut self, block: &Block<C, M>) -> Result<()> {
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

    async fn insert_batch(&mut self, batch: &[Block<C, M>]) -> Result<Cid> {
        let root = batch.last().ok_or(EmptyBatch)?.cid.clone();
        for block in batch {
            self._insert(block).await?;
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

#[async_trait]
impl<C: Codec, M: MultihashDigest> Store for MemStore<C, M>
where
    Ipld: Decode<C>,
{
    type Codec = C;
    type Multihash = M;
    const MAX_BLOCK_SIZE: usize = crate::MAX_BLOCK_SIZE;

    async fn pin(&self, cid: &Cid) -> Result<()> {
        self.local.write().await.pin(cid).await
    }

    async fn unpin(&self, cid: &Cid) -> Result<()> {
        self.local.write().await.unpin(cid).await
    }

    async fn get(&self, cid: Cid) -> Result<Block<C, M>> {
        self.local.read().await.get(cid).await
    }

    async fn insert_batch(&self, batch: &[Block<C, M>]) -> Result<Cid> {
        self.local.write().await.insert_batch(batch).await
    }

    async fn status(&self, cid: &Cid) -> Result<Option<Status>> {
        Ok(self.local.read().await.status(cid).await)
    }
}

#[async_trait]
impl<C: Codec, M: MultihashDigest> AliasStore for MemStore<C, M>
where
    Ipld: Decode<C>,
{
    async fn alias(&self, alias: &[u8], cid: &Cid) -> Result<()> {
        Ok(self.global.alias(alias, cid).await)
    }

    async fn unalias(&self, alias: &[u8]) -> Result<()> {
        Ok(self.global.unalias(alias).await)
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
    use crate::store::Store;

    fn create_block(ipld: &Ipld) -> Block<Multicodec, Multihash> {
        Block::encode(DagCborCodec, SHA2_256, ipld).unwrap()
    }

    #[async_std::test]
    async fn test_gc() -> Result<()> {
        let store = MemStore::<Multicodec, Multihash>::default();
        let a = store.insert(&create_block(&ipld!({ "a": [] }))).await?;
        let b = store.insert(&create_block(&ipld!({ "b": [&a] }))).await?;
        store.unpin(&a).await?;
        let c = store.insert(&create_block(&ipld!({ "c": [&a] }))).await?;
        assert!(store.status(&a).await?.is_some());
        assert!(store.status(&b).await?.is_some());
        assert!(store.status(&c).await?.is_some());
        store.unpin(&b).await?;
        assert!(store.status(&a).await?.is_some());
        assert!(store.status(&b).await?.is_none());
        assert!(store.status(&c).await?.is_some());
        store.unpin(&c).await?;
        assert!(store.status(&a).await?.is_none());
        assert!(store.status(&b).await?.is_none());
        assert!(store.status(&c).await?.is_none());
        Ok(())
    }

    #[async_std::test]
    async fn test_gc_2() -> Result<()> {
        let store = MemStore::<Multicodec, Multihash>::default();
        let a = store.insert(&create_block(&ipld!({ "a": [] }))).await?;
        let b = store.insert(&create_block(&ipld!({ "b": [&a] }))).await?;
        store.unpin(&a).await?;
        let c = store.insert(&create_block(&ipld!({ "b": [&a] }))).await?;
        assert!(store.status(&a).await?.is_some());
        assert!(store.status(&b).await?.is_some());
        assert!(store.status(&c).await?.is_some());
        store.unpin(&b).await?;
        assert!(store.status(&a).await?.is_some());
        assert!(store.status(&b).await?.is_some());
        assert!(store.status(&c).await?.is_some());
        store.unpin(&c).await?;
        assert!(store.status(&a).await?.is_none());
        assert!(store.status(&b).await?.is_none());
        assert!(store.status(&c).await?.is_none());
        Ok(())
    }

    #[async_std::test]
    async fn test_sync() -> Result<()> {
        let global = Arc::new(GlobalStore::<Multicodec, Multihash>::default());
        let local1 = MemStore::new(global.clone());
        let local2 = MemStore::new(global.clone());
        let a1 = create_block(&ipld!({ "a": 0 }));
        let b1 = create_block(&ipld!({ "b": 0 }));
        let c1 = create_block(&ipld!({ "c": [&a1.cid, &b1.cid] }));
        let b2 = create_block(&ipld!({ "b": 1 }));
        let c2 = create_block(&ipld!({ "c": [&a1.cid, &b2.cid] }));

        // insert alias
        let root1 = local1
            .insert_batch(&[a1.clone(), b1.clone(), c1.clone()])
            .await?;
        local1.alias(b"root", &root1).await?;

        assert!(local1.status(&a1.cid).await?.is_some());
        assert!(local1.status(&b1.cid).await?.is_some());
        assert!(local1.status(&c1.cid).await?.is_some());
        assert!(local1.status(&b2.cid).await?.is_none());
        assert!(local1.status(&c2.cid).await?.is_none());

        // resolve sync
        let root1p = local2.resolve(b"root").await?.unwrap();
        assert_eq!(root1, root1p);
        let c1p = local2.sync(root1p).await?;
        assert_eq!(c1, c1p);

        assert!(local2.status(&a1.cid).await?.is_some());
        assert!(local2.status(&b1.cid).await?.is_some());
        assert!(local2.status(&c1.cid).await?.is_some());
        assert!(local2.status(&b2.cid).await?.is_none());
        assert!(local2.status(&c2.cid).await?.is_none());

        // insert alias unpin
        let root2 = local2.insert_batch(&[b2.clone(), c2.clone()]).await?;
        local2.alias(b"root", &root2).await?;
        local2.unpin(&root1).await?;

        assert!(local2.status(&a1.cid).await?.is_some());
        assert!(local2.status(&b1.cid).await?.is_none());
        assert!(local2.status(&c1.cid).await?.is_none());
        assert!(local2.status(&b2.cid).await?.is_some());
        assert!(local2.status(&c2.cid).await?.is_some());

        // resolve sync unpin
        let root2p = local1.resolve(b"root").await?.unwrap();
        assert_eq!(root2, root2p);
        let c2p = local1.sync(root2p).await?;
        assert_eq!(c2, c2p);
        local1.unpin(&c1.cid).await?;

        assert!(local1.status(&a1.cid).await?.is_some());
        assert!(local1.status(&b1.cid).await?.is_none());
        assert!(local1.status(&c1.cid).await?.is_none());
        assert!(local1.status(&b2.cid).await?.is_some());
        assert!(local1.status(&c2.cid).await?.is_some());

        // unpin
        local1.unpin(&c2.cid).await?;

        assert!(local1.status(&a1.cid).await?.is_none());
        assert!(local1.status(&b1.cid).await?.is_none());
        assert!(local1.status(&c1.cid).await?.is_none());
        assert!(local1.status(&b2.cid).await?.is_none());
        assert!(local1.status(&c2.cid).await?.is_none());

        // unpin
        local2.unpin(&c2.cid).await?;

        assert!(local2.status(&a1.cid).await?.is_none());
        assert!(local2.status(&b1.cid).await?.is_none());
        assert!(local2.status(&c1.cid).await?.is_none());
        assert!(local2.status(&b2.cid).await?.is_none());
        assert!(local2.status(&c2.cid).await?.is_none());

        Ok(())
    }
}
