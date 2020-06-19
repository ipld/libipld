//! Reference implementation of the store traits.
use crate::block::Block;
use crate::cid::Cid;
use crate::encode_decode::EncodeDecodeIpld;
use crate::error::StoreError;
use crate::multihash::MultihashCode;
use crate::store::{AliasStore, ReadonlyStore, Store, StoreResult, Visibility};
use async_std::sync::{Arc, RwLock};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

#[derive(Default)]
struct InnerStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    blocks: HashMap<Cid<C, H>, Box<[u8]>>,
    refs: HashMap<Cid<C, H>, HashSet<Cid<C, H>>>,
    referers: HashMap<Cid<C, H>, isize>,
    pins: HashMap<Cid<C, H>, usize>,
}

impl<C, H> InnerStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy + Eq + EncodeDecodeIpld<H>,
    H: MultihashCode,
{
    /// Create a new empty `InnerStore`
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            refs: HashMap::new(),
            referers: HashMap::new(),
            pins: HashMap::new(),
        }
    }

    fn get(&self, cid: &Cid<C, H>) -> Result<Box<[u8]>, StoreError> {
        if let Some(data) = self.blocks.get(cid).cloned() {
            Ok(data)
        } else {
            Err(StoreError::BlockNotFound(cid.to_string()))
        }
    }

    fn add_referer(&mut self, cid: &Cid<C, H>, n: isize) {
        let (cid, referers) = self
            .referers
            .remove_entry(cid)
            .unwrap_or_else(|| (cid.clone(), 0));
        self.referers.insert(cid, referers + n);
    }

    fn insert(&mut self, cid: &Cid<C, H>, data: Box<[u8]>) -> Result<(), StoreError> {
        self.insert_block(cid, data)?;
        self.pin(cid);
        Ok(())
    }

    fn insert_block(&mut self, cid: &Cid<C, H>, data: Box<[u8]>) -> Result<(), StoreError> {
        if self.blocks.contains_key(cid) {
            return Ok(());
        }
        let ipld = crate::block::decode_ipld::<C, H>(cid, &data)
            .map_err(|e| StoreError::Other(Box::new(e)))?;
        let refs = crate::block::references(&ipld);
        for cid in &refs {
            self.add_referer(&cid, 1);
        }
        self.refs.insert(cid.clone(), refs);
        self.blocks.insert(cid.clone(), data);
        Ok(())
    }

    fn insert_batch(&mut self, batch: Vec<Block<C, H>>) -> Result<Cid<C, H>, StoreError> {
        let mut last_cid = None;
        for Block { cid, data } in batch.into_iter() {
            self.insert_block(&cid, data)?;
            last_cid = Some(cid);
        }
        Ok(last_cid.ok_or(StoreError::EmptyBatch)?)
    }

    fn pin(&mut self, cid: &Cid<C, H>) {
        let (cid, pins) = self
            .pins
            .remove_entry(cid)
            .unwrap_or_else(|| (cid.clone(), 0));
        self.pins.insert(cid, pins + 1);
    }

    fn unpin(&mut self, cid: &Cid<C, H>) -> Result<(), StoreError> {
        if let Some((cid, pins)) = self.pins.remove_entry(cid) {
            if pins > 1 {
                self.pins.insert(cid, pins - 1);
            } else {
                self.remove(&cid);
            }
        }
        Ok(())
    }

    fn remove(&mut self, cid: &Cid<C, H>) {
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
pub struct MemStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy,
    H: MultihashCode,
{
    inner: Arc<RwLock<InnerStore<C, H>>>,
    #[allow(clippy::type_complexity)]
    aliases: Arc<RwLock<HashMap<Box<[u8]>, Cid<C, H>>>>,
}
impl<C, H> MemStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy + Eq + EncodeDecodeIpld<H>,
    H: MultihashCode,
{
    /// Create a new empty `MemStore`
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerStore::<C, H>::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<C, H> ReadonlyStore<C, H> for MemStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy + Eq + Send + Sync + EncodeDecodeIpld<H>,
    H: MultihashCode,
{
    fn get<'a>(&'a self, cid: &'a Cid<C, H>) -> StoreResult<'a, Box<[u8]>> {
        Box::pin(async move { self.inner.read().await.get(cid) })
    }
}

impl<C, H> Store<C, H> for MemStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy + Eq + Send + Sync + EncodeDecodeIpld<H>,
    H: MultihashCode,
{
    fn insert<'a>(
        &'a self,
        cid: &'a Cid<C, H>,
        data: Box<[u8]>,
        _visibility: Visibility,
    ) -> StoreResult<'a, ()> {
        Box::pin(async move { self.inner.write().await.insert(cid, data) })
    }

    fn insert_batch<'a>(
        &'a self,
        batch: Vec<Block<C, H>>,
        _visibility: Visibility,
    ) -> StoreResult<'a, Cid<C, H>> {
        Box::pin(async move { self.inner.write().await.insert_batch(batch) })
    }

    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin<'a>(&'a self, cid: &'a Cid<C, H>) -> StoreResult<'a, ()> {
        Box::pin(async move { self.inner.write().await.unpin(cid) })
    }
}

impl<C, H> AliasStore<C, H> for MemStore<C, H>
where
    C: Into<u64> + TryFrom<u64> + Copy + Send + Sync,
    H: MultihashCode,
{
    fn alias<'a>(
        &'a self,
        alias: &'a [u8],
        cid: &'a Cid<C, H>,
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

    fn resolve<'a>(&'a self, alias: &'a [u8]) -> StoreResult<'a, Option<Cid<C, H>>> {
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
