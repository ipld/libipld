//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode};
use crate::error::{BlockNotFound, BlockTooLarge, Result};
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::store::{AliasStore, Op, Status, Store, Transaction};
use crate::MAX_BLOCK_SIZE;
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use core::borrow::Borrow;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct BlockInfo<C, H> {
    block: Block<C, H>,
    refs: HashSet<Cid>,
    status: Status,
}

impl<C, H> core::hash::Hash for BlockInfo<C, H> {
    fn hash<SH: core::hash::Hasher>(&self, hasher: &mut SH) {
        self.block.hash(hasher)
    }
}

impl<C, H> PartialEq for BlockInfo<C, H> {
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block
    }
}

impl<C, H> Eq for BlockInfo<C, H> {}

impl<C, H> Borrow<Cid> for BlockInfo<C, H> {
    fn borrow(&self) -> &Cid {
        self.block.borrow()
    }
}

impl<C: Codec, H: MultihashDigest> BlockInfo<C, H>
where
    Ipld: Decode<C>,
{
    pub fn new(block: Block<C, H>) -> Result<Self> {
        if block.data().len() > MAX_BLOCK_SIZE {
            return Err(BlockTooLarge(block.data().len()).into());
        }
        let refs = block.ipld()?.references();
        Ok(Self {
            block,
            refs,
            status: Default::default(),
        })
    }
}

enum MicroOp<C, H> {
    Insert(BlockInfo<C, H>),
    Pin(Cid),
    Unpin(Cid),
}

/// Models a network for testing.
pub struct GlobalStore<C, H> {
    blocks: RwLock<HashSet<Block<C, H>>>,
    aliases: RwLock<HashMap<Vec<u8>, Cid>>,
}

impl<C, H> Default for GlobalStore<C, H> {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            aliases: Default::default(),
        }
    }
}

impl<C: Codec, H: MultihashDigest> GlobalStore<C, H> {
    async fn get(&self, cid: &Cid) -> Result<Block<C, H>> {
        if let Some(block) = self.blocks.read().await.get(cid) {
            Ok(block.clone())
        } else {
            Err(BlockNotFound(cid.to_string()).into())
        }
    }

    async fn insert(&self, block: Block<C, H>) {
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

struct LocalStore<C, H> {
    global: Arc<GlobalStore<C, H>>,
    blocks: HashSet<BlockInfo<C, H>>,
}

impl<C, H> Default for LocalStore<C, H> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            blocks: Default::default(),
        }
    }
}

impl<C: Codec, H: MultihashDigest> LocalStore<C, H>
where
    Ipld: Decode<C>,
{
    /// Create a new empty `InnerStore`
    fn new(global: Arc<GlobalStore<C, H>>) -> Self {
        Self {
            global,
            ..Default::default()
        }
    }

    async fn get(&self, cid: &Cid) -> Result<Block<C, H>> {
        if let Some(info) = self.blocks.get(cid) {
            Ok(info.block.clone())
        } else {
            self.global.get(cid).await
        }
    }

    fn status(&self, cid: &Cid) -> Option<Status> {
        self.blocks.get(cid).map(|info| info.status)
    }

    fn blocks(&self) -> Vec<Cid> {
        self.blocks
            .iter()
            .map(|info| info.block.cid().clone())
            .collect()
    }

    fn verify_transaction(&self, tx: Transaction<C, H>) -> Result<Vec<MicroOp<C, H>>> {
        let mut touched = HashSet::with_capacity(tx.len());
        let mut micro_ops = Vec::with_capacity(tx.len());
        for op in tx {
            match op {
                Op::Insert(block) => {
                    if self.status(block.cid()).is_some() {
                        continue;
                    }
                    let info = BlockInfo::new(block)?;
                    for cid in &info.refs {
                        if !touched.contains(cid) && self.status(cid).is_none() {
                            return Err(BlockNotFound(cid.to_string()).into());
                        }
                        touched.insert(cid.clone());
                    }
                    touched.insert(info.block.cid().clone());
                    micro_ops.push(MicroOp::Insert(info));
                }
                Op::Pin(cid) => {
                    if !touched.contains(&cid) && self.status(&cid).is_none() {
                        return Err(BlockNotFound(cid.to_string()).into());
                    }
                    touched.insert(cid.clone());
                    micro_ops.push(MicroOp::Pin(cid));
                }
                Op::Unpin(cid) => {
                    if !touched.contains(&cid) && self.status(&cid).is_none() {
                        return Err(BlockNotFound(cid.to_string()).into());
                    }
                    touched.insert(cid.clone());
                    micro_ops.push(MicroOp::Unpin(cid));
                }
            }
        }
        Ok(micro_ops)
    }

    async fn commit(&mut self, tx: Transaction<C, H>) -> Result<()> {
        let mut dead = Vec::with_capacity(tx.len());
        let micro_ops = self.verify_transaction(tx)?;
        for op in micro_ops {
            match op {
                MicroOp::Insert(info) => {
                    for cid in &info.refs {
                        let mut info = self.blocks.take(cid).unwrap();
                        info.status.reference();
                        self.blocks.insert(info);
                    }
                    self.global.insert(info.block.clone()).await;
                    self.blocks.insert(info);
                }
                MicroOp::Pin(cid) => {
                    let mut info = self.blocks.take(&cid).unwrap();
                    info.status.pin();
                    self.blocks.insert(info);
                }
                MicroOp::Unpin(cid) => {
                    let mut info = self.blocks.take(&cid).unwrap();
                    info.status.unpin();
                    if info.status.is_dead() {
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
        for cid in &info.refs {
            if let Some(mut info) = self.blocks.take(cid) {
                info.status.unreference();
                self.blocks.insert(info);
            }
        }
        info.refs
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
pub struct MemStore<C, H> {
    global: Arc<GlobalStore<C, H>>,
    local: Arc<RwLock<LocalStore<C, H>>>,
}

impl<C, H> Default for MemStore<C, H> {
    fn default() -> Self {
        Self {
            global: Default::default(),
            local: Default::default(),
        }
    }
}

impl<C: Codec, H: MultihashDigest> MemStore<C, H>
where
    Ipld: Decode<C>,
{
    /// Create a new empty `MemStore`
    pub fn new(global: Arc<GlobalStore<C, H>>) -> Self {
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
impl<C: Codec, H: MultihashDigest> Store for MemStore<C, H>
where
    Ipld: Decode<C>,
{
    type Codec = C;
    type Multihash = H;
    const MAX_BLOCK_SIZE: usize = crate::MAX_BLOCK_SIZE;

    async fn get(&self, cid: Cid) -> Result<Block<C, H>> {
        self.local.read().await.get(&cid).await
    }

    async fn commit(&self, tx: Transaction<C, H>) -> Result<()> {
        self.local.write().await.commit(tx).await
    }

    async fn status(&self, cid: &Cid) -> Result<Option<Status>> {
        Ok(self.local.read().await.status(cid))
    }
}

#[async_trait]
impl<C: Codec, H: MultihashDigest> AliasStore for MemStore<C, H>
where
    Ipld: Decode<C>,
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
