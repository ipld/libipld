//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::{BlockNotFound, EmptyBatch, Result};
use crate::multihash::MultihashDigest;
use crate::store::{AliasStore, ReadonlyStore, Store, StoreResult};
use async_std::sync::{Arc, RwLock};
use core::marker::PhantomData;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct InnerStore {
    blocks: HashMap<Cid, Box<[u8]>>,
    refs: HashMap<Cid, HashSet<Cid>>,
    referers: HashMap<Cid, isize>,
    pins: HashMap<Cid, usize>,
}

impl InnerStore {
    /// Create a new empty `InnerStore`
    pub fn new() -> Self {
        Self::default()
    }

    fn get<C: Codec, M: MultihashDigest>(&self, cid: Cid) -> Result<Block<C, M>> {
        if let Some(data) = self.blocks.get(&cid).cloned() {
            Ok(Block::new(cid, data))
        } else {
            Err(BlockNotFound(cid.to_string()).into())
        }
    }

    fn add_referer(&mut self, cid: &Cid, n: isize) {
        let (cid, referers) = self
            .referers
            .remove_entry(cid)
            .unwrap_or_else(|| (cid.clone(), 0));
        self.referers.insert(cid, referers + n);
    }

    fn insert<C, M>(&mut self, block: &Block<C, M>) -> Result<()>
    where
        C: Codec,
        M: MultihashDigest,
    {
        self.insert_block(&block)?;
        self.pin(&block.cid);
        Ok(())
    }

    fn insert_block<C, M>(&mut self, block: &Block<C, M>) -> Result<()>
    where
        C: Codec,
        M: MultihashDigest,
    {
        if self.blocks.contains_key(&block.cid) {
            return Ok(());
        }
        let ipld = block.decode_ipld()?;
        let refs = ipld.references();
        for cid in &refs {
            self.add_referer(cid, 1);
        }
        self.refs.insert(block.cid.clone(), refs);
        self.blocks.insert(block.cid.clone(), block.data.clone());
        Ok(())
    }

    fn insert_batch<C, M>(&mut self, batch: &[Block<C, M>]) -> Result<Cid>
    where
        C: Codec,
        M: MultihashDigest,
    {
        let mut last_cid = None;
        for block in batch {
            self.insert_block(block)?;
            last_cid = Some(block.cid.clone());
        }
        Ok(last_cid.ok_or(EmptyBatch)?)
    }

    fn pin(&mut self, cid: &Cid) {
        let (cid, pins) = self
            .pins
            .remove_entry(cid)
            .unwrap_or_else(|| (cid.clone(), 0));
        self.pins.insert(cid, pins + 1);
    }

    fn unpin(&mut self, cid: &Cid) -> Result<()> {
        if let Some((cid, pins)) = self.pins.remove_entry(cid) {
            if pins > 1 {
                self.pins.insert(cid, pins - 1);
            } else {
                self.remove(&cid);
            }
        }
        Ok(())
    }

    fn remove(&mut self, cid: &Cid) {
        let pins = self.pins.get(&cid).cloned().unwrap_or_default();
        let referers = self.referers.get(&cid).cloned().unwrap_or_default();
        if referers < 1 && pins < 1 {
            self.blocks.remove(&cid);
            let refs = self.refs.remove(&cid).unwrap();
            for cid in &refs {
                self.add_referer(cid, -1);
                self.remove(cid);
            }
        }
    }
}

/// A memory backed store
#[derive(Clone, Default)]
pub struct MemStore<C: Codec, M: MultihashDigest> {
    _marker: PhantomData<(C, M)>,
    inner: Arc<RwLock<InnerStore>>,
    #[allow(clippy::type_complexity)]
    aliases: Arc<RwLock<HashMap<Box<[u8]>, Cid>>>,
}

impl<C: Codec, M: MultihashDigest> MemStore<C, M> {
    /// Create a new empty `MemStore`
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            inner: Arc::new(RwLock::new(InnerStore::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<C: Codec, M: MultihashDigest> ReadonlyStore for MemStore<C, M> {
    type Codec = C;
    type Multihash = M;

    fn get<'a>(&'a self, cid: Cid) -> StoreResult<'a, Block<C, M>> {
        Box::pin(async move { self.inner.read().await.get(cid) })
    }
}

impl<C: Codec, M: MultihashDigest> Store for MemStore<C, M> {
    fn insert<'a>(&'a self, block: &'a Block<C, M>) -> StoreResult<'a, ()> {
        Box::pin(async move { self.inner.write().await.insert(block) })
    }

    fn insert_batch<'a>(&'a self, batch: &'a [Block<C, M>]) -> StoreResult<'a, Cid> {
        Box::pin(async move { self.inner.write().await.insert_batch(batch) })
    }

    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move { self.inner.write().await.unpin(cid) })
    }
}

impl<C: Codec, M: MultihashDigest> AliasStore for MemStore<C, M> {
    fn alias<'a>(&'a self, alias: &'a [u8], block: &'a Block<C, M>) -> StoreResult<'a, ()> {
        Box::pin(async move {
            self.aliases
                .write()
                .await
                .insert(alias.to_vec().into_boxed_slice(), block.cid.clone());
            Ok(())
        })
    }

    fn unalias<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, ()> {
        Box::pin(async move {
            self.aliases.write().await.remove(alias);
            Ok(())
        })
    }

    fn resolve<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, Option<Cid>> {
        Box::pin(async move { Ok(self.aliases.read().await.get(alias).cloned()) })
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

    async fn get<S: ReadonlyStore>(store: &S, cid: &Cid) -> Option<Ipld> {
        let block = match store.get(cid.clone()).await {
            Ok(block) => block,
            Err(e) if e.downcast_ref::<BlockNotFound>().is_some() => return None,
            Err(e) => Err(e).unwrap(),
        };
        Some(block.decode_ipld().unwrap())
    }

    async fn insert<S: Store>(store: &S, ipld: &Ipld) -> Cid
    where
        S::Codec: From<DagCborCodec>,
    {
        let block = Block::<DagCborCodec, _>::encode_ipld(DagCborCodec, SHA2_256, ipld).unwrap();
        let block = Block::try_from_block(block).unwrap();
        store.insert(&block).await.unwrap();
        block.cid
    }

    #[async_std::test]
    async fn test_gc() {
        let store = MemStore::<DagCborCodec, Multihash>::new();
        let a = insert(&store, &ipld!({ "a": [] })).await;
        let b = insert(&store, &ipld!({ "b": [&a] })).await;
        store.unpin(&a).await.unwrap();
        let c = insert(&store, &ipld!({ "c": [&a] })).await;
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_some());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&b).await.unwrap();
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_none());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&c).await.unwrap();
        assert!(get(&store, &a).await.is_none());
        assert!(get(&store, &b).await.is_none());
        assert!(get(&store, &c).await.is_none());
    }

    #[async_std::test]
    async fn test_gc_2() {
        let store = MemStore::<DagCborCodec, Multihash>::new();
        let a = insert(&store, &ipld!({ "a": [] })).await;
        let b = insert(&store, &ipld!({ "b": [&a] })).await;
        store.unpin(&a).await.unwrap();
        let c = insert(&store, &ipld!({ "b": [&a] })).await;
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_some());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&b).await.unwrap();
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_some());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&c).await.unwrap();
        assert!(get(&store, &a).await.is_none());
        assert!(get(&store, &b).await.is_none());
        assert!(get(&store, &c).await.is_none());
    }
}
