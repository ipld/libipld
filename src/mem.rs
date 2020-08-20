//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::Codec;
use crate::error::Error;
use crate::error::StoreError;
use crate::multihash::MultihashDigest;
use crate::store::{AliasStore, ReadonlyStore, Store, StoreResult, Visibility};
use async_std::sync::{Arc, RwLock};
use core::convert::TryFrom;
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

    fn get<C, M>(&self, cid: Cid) -> Result<Block<C, M>, StoreError> {
        if let Some(data) = self.blocks.get(&cid).cloned() {
            Ok(Block {
                _marker: PhantomData,
                cid,
                data,
            })
        } else {
            Err(StoreError::BlockNotFound(cid.to_string()))
        }
    }

    fn add_referer(&mut self, cid: &Cid, n: isize) {
        let (cid, referers) = self
            .referers
            .remove_entry(cid)
            .unwrap_or_else(|| (cid.clone(), 0));
        self.referers.insert(cid, referers + n);
    }

    fn insert<C, M>(&mut self, block: &Block<C, M>) -> Result<(), StoreError>
    where
        C: Codec + TryFrom<u64, Error = Error>,
        M: MultihashDigest,
    {
        self.insert_block(&block)?;
        self.pin(&block.cid);
        Ok(())
    }

    fn insert_block<C, M>(&mut self, block: &Block<C, M>) -> Result<(), StoreError>
    where
        C: Codec + TryFrom<u64, Error = Error>,
        M: MultihashDigest,
    {
        if self.blocks.contains_key(&block.cid) {
            return Ok(());
        }
        let ipld = block
            .decode_ipld()
            .map_err(|e| StoreError::Other(e.into()))?;
        let refs = ipld.references();
        for cid in &refs {
            self.add_referer(&block.cid, 1);
        }
        self.refs.insert(block.cid.clone(), refs);
        self.blocks.insert(block.cid.clone(), block.data.clone());
        Ok(())
    }

    fn insert_batch<C, M>(&mut self, batch: &[Block<C, M>]) -> Result<Cid, StoreError>
    where
        C: Codec + TryFrom<u64, Error = Error>,
        M: MultihashDigest,
    {
        let mut last_cid = None;
        for block in batch {
            self.insert_block(block)?;
            last_cid = Some(block.cid.clone());
        }
        Ok(last_cid.ok_or(StoreError::EmptyBatch)?)
    }

    fn pin(&mut self, cid: &Cid) {
        let (cid, pins) = self
            .pins
            .remove_entry(cid)
            .unwrap_or_else(|| (cid.clone(), 0));
        self.pins.insert(cid, pins + 1);
    }

    fn unpin(&mut self, cid: &Cid) -> Result<(), StoreError> {
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
pub struct MemStore {
    inner: Arc<RwLock<InnerStore>>,
    #[allow(clippy::type_complexity)]
    aliases: Arc<RwLock<HashMap<Box<[u8]>, Cid>>>,
}

impl MemStore {
    /// Create a new empty `MemStore`
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerStore::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ReadonlyStore for MemStore {
    fn get<'a, C: Codec, M: MultihashDigest>(&'a self, cid: Cid) -> StoreResult<'a, Block<C, M>> {
        Box::pin(async move { self.inner.read().await.get(cid) })
    }
}

impl Store for MemStore {
    fn insert<'a, C: Codec, M: MultihashDigest>(
        &'a self,
        block: &'a Block<C, M>,
        _visibility: Visibility,
    ) -> StoreResult<'a, ()> {
        Box::pin(async move { self.inner.write().await.insert(block) })
    }

    fn insert_batch<'a, C: Codec, M: MultihashDigest>(
        &'a self,
        batch: &'a [Block<C, M>],
        _visibility: Visibility,
    ) -> StoreResult<'a, Cid> {
        Box::pin(async move { self.inner.write().await.insert_batch(batch) })
    }

    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move { self.inner.write().await.unpin(cid) })
    }
}

impl AliasStore for MemStore {
    fn alias<'a>(
        &'a self,
        alias: &'a [u8],
        cid: &'a Cid,
        _visibility: Visibility,
    ) -> StoreResult<'a, ()> {
        Box::pin(async move {
            self.aliases
                .write()
                .await
                .insert(alias.to_vec().into_boxed_slice(), cid.clone());
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
    use crate::block::{decode, encode, Block};
    use crate::cbor::DagCborCodec;
    use crate::codec::{Cid, IpldCodec};
    use crate::ipld;
    use crate::ipld::Ipld;
    use crate::multihash::{Code as HCode, Sha2_256};
    use crate::store::{Store, Visibility};

    async fn get<S: ReadonlyStore>(store: &S, cid: &Cid) -> Option<Ipld> {
        let bytes = match store.get(cid).await {
            Ok(bytes) => bytes,
            Err(StoreError::BlockNotFound { .. }) => return None,
            Err(e) => Err(e).unwrap(),
        };
        Some(decode::<IpldCodec, HCode, DagCborCodec, Ipld>(cid, &bytes).unwrap())
    }

    async fn insert<S: Store>(store: &S, ipld: &Ipld) -> Cid {
        let Block { cid, data } =
            encode::<IpldCodec, HCode, DagCborCodec, Sha2_256, Ipld>(ipld).unwrap();
        store.insert(&cid, data, Visibility::Public).await.unwrap();
        cid
    }

    #[async_std::test]
    async fn test_gc() {
        let store = MemStore::new();
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
        let store = MemStore::new();
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
