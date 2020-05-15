//! Reference implementation of the store traits.
use crate::cid::Cid;
use crate::error::{Error, StoreError};
use crate::store::{AliasStore, ReadonlyStore, Store, StoreResult, Visibility};
use async_std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use async_std::task;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct UpgradeableRwLock<T> {
    lock: Mutex<()>,
    rwlock: RwLock<T>,
}

struct UpgradeableRwLockReadGuard<'a, T> {
    lock: MutexGuard<'a, ()>,
    guard: RwLockReadGuard<'a, T>,
}

impl<'a, T> core::ops::Deref for UpgradeableRwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

struct UpgradeableRwLockWriteGuard<'a, T> {
    #[allow(unused)]
    lock: MutexGuard<'a, ()>,
    guard: RwLockWriteGuard<'a, T>,
}

impl<'a, T> core::ops::Deref for UpgradeableRwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<'a, T> core::ops::DerefMut for UpgradeableRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<T> UpgradeableRwLock<T> {
    async fn write(&self) -> UpgradeableRwLockWriteGuard<'_, T> {
        let lock = self.lock.lock().await;
        let guard = self.rwlock.write().await;
        UpgradeableRwLockWriteGuard { lock, guard }
    }

    async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.rwlock.read().await
    }

    async fn upgradeable_read(&self) -> UpgradeableRwLockReadGuard<'_, T> {
        let lock = self.lock.lock().await;
        let guard = self.rwlock.read().await;
        UpgradeableRwLockReadGuard { lock, guard }
    }

    async fn upgrade<'a>(
        &'a self,
        guard: UpgradeableRwLockReadGuard<'a, T>,
    ) -> UpgradeableRwLockWriteGuard<'a, T> {
        let UpgradeableRwLockReadGuard { lock, guard } = guard;
        drop(guard);
        let guard = self.rwlock.write().await;
        UpgradeableRwLockWriteGuard { lock, guard }
    }
}

#[derive(Default)]
struct InnerStore {
    blocks: HashMap<Cid, Box<[u8]>>,
    roots: HashMap<Cid, usize>,
}

impl ReadonlyStore for InnerStore {
    fn get<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Box<[u8]>> {
        Box::pin(async move {
            if let Some(data) = self.blocks.get(cid).cloned() {
                Ok(data)
            } else {
                Err(StoreError::BlockNotFound(cid.clone()))
            }
        })
    }
}

struct Gc {
    inner: Arc<UpgradeableRwLock<InnerStore>>,
}

impl Gc {
    fn new(store: &MemStore) -> Self {
        Self {
            inner: store.inner.clone(),
        }
    }

    async fn gc(self) -> Result<(), Error> {
        // hold read lock on roots
        let inner = self.inner.upgradeable_read().await;

        let roots: HashSet<Cid> = inner.roots.iter().map(|(k, _)| k.clone()).collect();
        // TODO: avoid decoding blocks by storing the references on insert
        let live = crate::gc::recursive_references(&*inner, roots).await?;
        let cids: HashSet<Cid> = inner.blocks.iter().map(|(k, _)| k.clone()).collect();

        // TODO: allow temporary inserts, useful with a large block store.

        // atomically upgrade lock
        let mut inner = self.inner.upgrade(inner).await;

        // TODO: add closure of temporary inserts to live

        // remove dead blocks
        //
        // in a file backed implementation these need to be sorted topologically to prevent
        // corruption if the process crashes or is terminated. pinning acquires a file lock
        // on the block, preventing the garbage collector from removing the block.
        for cid in cids.difference(&live) {
            inner.blocks.remove(cid);
        }
        Ok(())
    }
}

/// A memory backed store
#[derive(Default)]
pub struct MemStore {
    inner: Arc<UpgradeableRwLock<InnerStore>>,
    aliases: RwLock<HashMap<Box<[u8]>, Cid>>,
}

impl ReadonlyStore for MemStore {
    fn get<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, Box<[u8]>> {
        Box::pin(async move { self.inner.read().await.get(cid).await })
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
            let mut inner = self.inner.write().await;
            inner.blocks.insert(cid.clone(), data);
            let count = inner.roots.get(cid).cloned().unwrap_or_default() + 1;
            inner.roots.insert(cid.clone(), count);
            Ok(())
        })
    }

    fn flush(&self) -> StoreResult<'_, ()> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin<'a>(&'a self, cid: &'a Cid) -> StoreResult<'a, ()> {
        Box::pin(async move {
            let mut inner = self.inner.write().await;
            if let Some((cid, count)) = inner.roots.remove_entry(cid) {
                if count > 1 {
                    inner.roots.insert(cid, count - 1);
                } else {
                    task::spawn(Gc::new(self).gc());
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{decode, encode, Block};
    use crate::cbor::DagCbor;
    use crate::cid::Cid;
    use crate::ipld;
    use crate::ipld::Ipld;
    use crate::multihash::Sha2_256;
    use crate::store::{Store, Visibility};
    use std::time::Duration;

    async fn get<S: ReadonlyStore>(store: &S, cid: &Cid) -> Option<Ipld> {
        let bytes = match store.get(cid).await {
            Ok(bytes) => bytes,
            Err(StoreError::BlockNotFound { .. }) => return None,
            Err(e) => Err(e).unwrap(),
        };
        Some(decode::<DagCbor, Ipld>(cid, &bytes).unwrap())
    }

    async fn insert<S: Store>(store: &S, ipld: &Ipld) -> Cid {
        let Block { cid, data } = encode::<DagCbor, Sha2_256, Ipld>(ipld).unwrap();
        store.insert(&cid, data, Visibility::Public).await.unwrap();
        cid
    }

    #[async_std::test]
    async fn test_upgrade() {
        let lock: Arc<UpgradeableRwLock<bool>> = Default::default();
        let lock2 = lock.clone();
        let guard = lock.upgradeable_read().await;
        assert_eq!(*guard, false);
        task::spawn(async move {
            let guard = lock2.read().await;
            assert_eq!(*guard, false);
            task::sleep(Duration::from_millis(100)).await;
            assert_eq!(*guard, false);
            drop(guard);
            task::sleep(Duration::from_millis(100)).await;
            let guard = lock2.read().await;
            assert_eq!(*guard, true);
        });
        task::sleep(Duration::from_millis(10)).await;
        let mut guard = lock.upgrade(guard).await;
        *guard = true;
    }

    #[async_std::test]
    async fn test_gc() {
        let store = MemStore::default();
        let a = insert(&store, &ipld!({"a": []})).await;
        let b = insert(&store, &ipld!({ "b": [&a] })).await;
        store.unpin(&a).await.unwrap();
        let c = insert(&store, &ipld!({ "c": [&a] })).await;
        task::sleep(Duration::from_millis(100)).await;
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_some());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&b).await.unwrap();
        task::sleep(Duration::from_millis(100)).await;
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_none());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&c).await.unwrap();
        task::sleep(Duration::from_millis(100)).await;
        assert!(get(&store, &a).await.is_none());
        assert!(get(&store, &b).await.is_none());
        assert!(get(&store, &c).await.is_none());
    }

    #[async_std::test]
    async fn test_gc_2() {
        let store = MemStore::default();
        let a = insert(&store, &ipld!({"a": []})).await;
        let b = insert(&store, &ipld!({ "b": [&a] })).await;
        store.unpin(&a).await.unwrap();
        let c = insert(&store, &ipld!({ "b": [&a] })).await;
        task::sleep(Duration::from_millis(100)).await;
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_some());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&b).await.unwrap();
        task::sleep(Duration::from_millis(100)).await;
        assert!(get(&store, &a).await.is_some());
        assert!(get(&store, &b).await.is_some());
        assert!(get(&store, &c).await.is_some());
        store.unpin(&c).await.unwrap();
        task::sleep(Duration::from_millis(100)).await;
        assert!(get(&store, &a).await.is_none());
        assert!(get(&store, &b).await.is_none());
        assert!(get(&store, &c).await.is_none());
    }
}
