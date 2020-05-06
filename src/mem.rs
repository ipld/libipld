//! Reference implementation of the store traits.
use crate::cid::Cid;
use crate::error::StoreError;
use crate::store::{AliasStore, ReadonlyStore, Store, StoreResult, Visibility};
use async_std::sync::RwLock;
use std::collections::HashMap;

/// A memory backed store
#[derive(Default)]
pub struct MemStore {
    blocks: RwLock<HashMap<Cid, Box<[u8]>>>,
    aliases: RwLock<HashMap<Box<[u8]>, Cid>>,
}

impl ReadonlyStore for MemStore {
    fn get<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Box<[u8]>> {
        Box::pin(async move {
            if let Some(data) = self.blocks.read().await.get(cid).cloned() {
                Ok(data)
            } else {
                Err(StoreError::BlockNotFound(cid.clone()))
            }
        })
    }
}

impl Store for MemStore {
    fn insert<'a>(
        &'a self,
        cid: &'a Cid,
        data: Box<[u8]>,
        _visibility: Visibility,
    ) -> StoreResult<'a, ()> {
        Box::pin(async move {
            self.blocks.write().await.insert(cid.clone(), data);
            Ok(())
        })
    }

    /// Flushes the write buffer.
    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }

    /// Marks a block ready for garbage collection.
    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move {
            self.blocks.write().await.remove(cid);
            Ok(())
        })
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
