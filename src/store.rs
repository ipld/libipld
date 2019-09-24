//! Traits for implementing a block store.
use crate::block::{create_cbor_block, decode_cbor, decode_ipld, validate};
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::error::Result;
use crate::hash::Hash;
use crate::ipld::{Cid, Ipld};
use async_std::sync::RwLock;
use async_trait::async_trait;
use core::hash::{BuildHasher, Hasher};
use core::ops::Deref;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// Implementable by ipld storage backends.
#[async_trait]
pub trait Store: Send + Sync {
    /// Returns the block with cid.
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>>;
    /// Writes the block with cid.
    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()>;
    /// Flushes the write buffer.
    async fn flush(&self) -> Result<()>;
    /// Gc's unused blocks.
    async fn gc(&self) -> Result<()>;

    /// Pin a block.
    async fn pin(&self, cid: &Cid) -> Result<()>;
    /// Unpin a block.
    async fn unpin(&self, cid: &Cid) -> Result<()>;
    /// Create an indirect user managed pin.
    async fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()>;

    /// Write a link to a block.
    async fn write_link(&self, label: &str, cid: &Cid) -> Result<()>;
    /// Read a link to a block.
    async fn read_link(&self, label: &str) -> Result<Option<Cid>>;
    /// Remove link to a block.
    async fn remove_link(&self, label: &str) -> Result<()>;
}

#[async_trait]
impl<TStore: Store> Store for Arc<TStore> {
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        self.deref().read(cid).await
    }

    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        self.deref().write(cid, data).await
    }

    async fn flush(&self) -> Result<()> {
        self.deref().flush().await
    }

    async fn gc(&self) -> Result<()> {
        self.deref().gc().await
    }

    async fn pin(&self, cid: &Cid) -> Result<()> {
        self.deref().pin(cid).await
    }

    async fn unpin(&self, cid: &Cid) -> Result<()> {
        self.deref().unpin(cid).await
    }

    async fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()> {
        self.deref().autopin(cid, auto_path).await
    }

    async fn write_link(&self, label: &str, cid: &Cid) -> Result<()> {
        self.deref().write_link(label, cid).await
    }

    async fn read_link(&self, label: &str) -> Result<Option<Cid>> {
        self.deref().read_link(label).await
    }

    async fn remove_link(&self, label: &str) -> Result<()> {
        self.deref().remove_link(label).await
    }
}

/// Ipld extension trait.
#[async_trait]
pub trait StoreIpldExt {
    /// Reads the block with cid and decodes it to ipld.
    async fn read_ipld(&self, cid: &Cid) -> Result<Option<Ipld>>;
}

#[async_trait]
impl<T: Store> StoreIpldExt for T {
    async fn read_ipld(&self, cid: &Cid) -> Result<Option<Ipld>> {
        if let Some(data) = self.read(cid).await? {
            let ipld = decode_ipld(cid, &data).await?;
            return Ok(Some(ipld));
        }
        Ok(None)
    }
}

/// Cbor extension trait.
#[async_trait]
pub trait StoreCborExt {
    /// Reads the block with cid and decodes it to cbor.
    async fn read_cbor<C: ReadCbor + Send>(&self, cid: &Cid) -> Result<Option<C>>;

    /// Writes a block using the cbor codec.
    async fn write_cbor<H: Hash, C: WriteCbor + Send + Sync>(&self, c: &C) -> Result<Cid>;
}

#[async_trait]
impl<T: Store> StoreCborExt for T {
    async fn read_cbor<C: ReadCbor + Send>(&self, cid: &Cid) -> Result<Option<C>> {
        if let Some(data) = self.read(cid).await? {
            let cbor = decode_cbor::<C>(cid, &data).await?;
            return Ok(Some(cbor));
        }
        Ok(None)
    }

    async fn write_cbor<H: Hash, C: WriteCbor + Send + Sync>(&self, c: &C) -> Result<Cid> {
        let (cid, data) = create_cbor_block::<H, C>(c).await?;
        self.write(&cid, data).await?;
        Ok(cid)
    }
}

#[derive(Default)]
struct BuildCidHasher;

impl BuildHasher for BuildCidHasher {
    type Hasher = CidHasher;

    fn build_hasher(&self) -> Self::Hasher {
        CidHasher(None)
    }
}

struct CidHasher(Option<u64>);

impl Hasher for CidHasher {
    fn finish(&self) -> u64 {
        self.0.unwrap()
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!();
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = Some(i);
    }
}

/// A memory backed store
#[derive(Default)]
pub struct MemStore {
    blocks: RwLock<HashMap<Cid, Box<[u8]>, BuildCidHasher>>,
    pins: RwLock<HashSet<Cid, BuildCidHasher>>,
    links: RwLock<HashMap<String, Cid>>,
}

#[async_trait]
impl Store for MemStore {
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        Ok(self.blocks.read().await.get(cid).cloned())
    }

    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        self.blocks.write().await.insert(cid.clone(), data);
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }

    async fn gc(&self) -> Result<()> {
        let pins = self.pins.read().await;
        let roots = pins.iter().map(Clone::clone).collect();
        let blocks = self
            .blocks
            .read()
            .await
            .iter()
            .map(|(cid, _)| cid.clone())
            .collect();
        let dead = crate::gc::dead_paths(self, blocks, roots).await?;
        for cid in dead {
            self.blocks.write().await.remove(&cid);
        }
        Ok(())
    }

    async fn pin(&self, cid: &Cid) -> Result<()> {
        self.pins.write().await.insert(cid.clone());
        Ok(())
    }

    async fn unpin(&self, cid: &Cid) -> Result<()> {
        self.pins.write().await.remove(&cid);
        Ok(())
    }

    async fn autopin(&self, cid: &Cid, _: &Path) -> Result<()> {
        self.pin(cid).await
    }

    async fn write_link(&self, link: &str, cid: &Cid) -> Result<()> {
        self.links
            .write()
            .await
            .insert(link.to_string(), cid.clone());
        Ok(())
    }

    async fn read_link(&self, link: &str) -> Result<Option<Cid>> {
        Ok(self.links.read().await.get(link).cloned())
    }

    async fn remove_link(&self, link: &str) -> Result<()> {
        self.links.write().await.remove(link);
        Ok(())
    }
}

/// A buffered store.
pub struct BufStore<TStore: Store = MemStore> {
    store: TStore,
    cache: RwLock<HashMap<Cid, Box<[u8]>, BuildCidHasher>>,
    buffer: RwLock<HashMap<Cid, Box<[u8]>, BuildCidHasher>>,
    //pins: RwLock<HashSet<Cid, BuildCidHasher>>,
    //unpins: RwLock<HashSet<Cid, BuildCidHasher>>,
}

impl<TStore: Store> BufStore<TStore> {
    /// Creates a new block store.
    pub fn new(store: TStore, cache_cap: usize, buffer_cap: usize) -> Self {
        Self {
            store,
            cache: RwLock::new(HashMap::with_capacity_and_hasher(cache_cap, BuildCidHasher)),
            buffer: RwLock::new(HashMap::with_capacity_and_hasher(
                buffer_cap,
                BuildCidHasher,
            )),
            //pins: Default::default(),
            //unpins: Default::default(),
        }
    }
}

#[async_trait]
impl<TStore: Store> Store for BufStore<TStore> {
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        let cached = self.cache.read().await.get(cid).cloned();
        if let Some(data) = cached {
            return Ok(Some(data));
        }
        let fresh = self.store.read(cid).await?;
        if let Some(ref data) = fresh {
            validate(cid, &data)?;
            self.cache.write().await.insert(cid.clone(), data.clone());
        }
        Ok(fresh)
    }

    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        self.cache.write().await.insert(cid.clone(), data.clone());
        self.buffer.write().await.insert(cid.clone(), data);
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        // TODO gc writes
        for (cid, data) in self.buffer.write().await.drain() {
            self.store.write(&cid, data).await?;
        }
        self.store.flush().await?;
        Ok(())
    }

    async fn gc(&self) -> Result<()> {
        self.store.gc().await
    }

    async fn pin(&self, cid: &Cid) -> Result<()> {
        //self.pins.write().await.insert(cid.clone());
        self.store.pin(cid).await?;
        Ok(())
    }

    async fn unpin(&self, cid: &Cid) -> Result<()> {
        //self.unpins.write().await.insert(cid.clone());
        self.store.unpin(cid).await?;
        Ok(())
    }

    async fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()> {
        self.store.autopin(cid, auto_path).await
    }

    async fn write_link(&self, label: &str, cid: &Cid) -> Result<()> {
        self.store.write_link(label, cid).await
    }

    async fn read_link(&self, label: &str) -> Result<Option<Cid>> {
        self.store.read_link(label).await
    }

    async fn remove_link(&self, label: &str) -> Result<()> {
        self.store.remove_link(label).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{create_cbor_block, create_raw_block};
    use crate::DefaultHash as H;
    use async_std::task;
    use core::future::Future;
    use futures::join;
    use model::*;

    fn create_block_raw(n: usize) -> (Cid, Box<[u8]>) {
        let data = n.to_ne_bytes().to_vec().into_boxed_slice();
        create_raw_block::<H>(data).unwrap()
    }

    async fn create_block_ipld(ipld: &Ipld) -> (Cid, Box<[u8]>) {
        create_cbor_block::<H, _>(ipld).await.unwrap()
    }

    #[test]
    fn test_obj() {
        let store = MemStore::default();
        let _ = &store as &dyn Store;
        let store = Arc::new(store);
        let _ = &store as &dyn Store;

        let store = MemStore::default();
        let store = BufStore::new(store, 16, 16);
        let _ = &store as &dyn Store;
        let store = Arc::new(store);
        let _ = &store as &dyn Store;
    }

    async fn run_gc_no_pin(store: &dyn Store) {
        let (cid_0, data_0) = create_block_raw(0);
        store.write(&cid_0, data_0).await.unwrap();
        store.flush().await.unwrap();
        store.gc().await.unwrap();
        let data_0_2 = store.read(&cid_0).await.unwrap();
        assert!(data_0_2.is_none());
    }

    #[test]
    fn test_gc_no_pin() {
        let store = MemStore::default();
        task::block_on(run_gc_no_pin(&store));
    }

    async fn run_gc_pin(store: &dyn Store) {
        let (cid_0, data_0) = create_block_raw(0);
        store.write(&cid_0, data_0.clone()).await.unwrap();
        store.pin(&cid_0).await.unwrap();
        store.flush().await.unwrap();
        store.gc().await.unwrap();
        let data_0_2 = store.read(&cid_0).await.unwrap();
        assert_eq!(data_0_2, Some(data_0));
    }

    #[test]
    fn test_gc_pin() {
        let store = MemStore::default();
        task::block_on(run_gc_pin(&store));
    }

    async fn run_gc_pin_leaf(store: &dyn Store) {
        let (cid_0, data_0) = create_block_raw(0);
        let ipld = Ipld::Link(cid_0.clone());
        let (cid_1, data_1) = create_block_ipld(&ipld).await;
        store.write(&cid_0, data_0.clone()).await.unwrap();
        store.write(&cid_1, data_1.clone()).await.unwrap();
        store.pin(&cid_1).await.unwrap();
        store.flush().await.unwrap();
        store.gc().await.unwrap();
        let data_0_2 = store.read(&cid_0).await.unwrap();
        assert_eq!(data_0_2, Some(data_0));
    }

    #[test]
    fn test_gc_pin_leaf() {
        let store = MemStore::default();
        task::block_on(run_gc_pin_leaf(&store));
    }

    fn join<T>(f1: impl Future<Output = Result<T>>, f2: impl Future<Output = Result<T>>) -> (T, T) {
        task::block_on(async {
            let f1_u = async { f1.await.unwrap() };
            let f2_u = async { f2.await.unwrap() };
            join!(f1_u, f2_u)
        })
    }

    #[test]
    fn mem_buf_store_eqv() {
        const LEN: usize = 4;
        let blocks: Vec<_> = (0..LEN).into_iter().map(create_block_raw).collect();
        model! {
            Model => let mem_store = MemStore::default(),
            Implementation => let buf_store = BufStore::new(MemStore::default(), 16, 16),
            Read(usize)(i in 0..LEN) => {
                let (cid, _) = &blocks[i];
                let mem = mem_store.read(cid);
                let buf = buf_store.read(cid);
                let (mem, buf) = join(mem, buf);
                // Element can be in cache after gc.
                if !(mem.is_none() && buf.is_some()) {
                    assert_eq!(mem, buf);
                }
            },
            Write(usize)(i in 0..LEN) => {
                let (cid, data) = &blocks[i];
                let mem = mem_store.write(cid, data.clone());
                let buf = buf_store.write(cid, data.clone());
                join(mem, buf);
            },
            Flush(usize)(_ in 0..LEN) => {
                let mem = mem_store.flush();
                let buf = buf_store.flush();
                join(mem, buf);
            },
            Gc(usize)(_ in 0..LEN) => {
                let mem = mem_store.gc();
                let buf = buf_store.gc();
                join(mem, buf);
            },
            Pin(usize)(i in 0..LEN) => {
                let (cid, _) = &blocks[i];
                let mem = mem_store.pin(&cid);
                let buf = buf_store.pin(&cid);
                join(mem, buf);
            },
            Unpin(usize)(i in 0..LEN) => {
                let (cid, _) = &blocks[i];
                let mem = mem_store.unpin(&cid);
                let buf = buf_store.unpin(&cid);
                join(mem, buf);
            }
        }
    }

    macro_rules! linearizable_store {
        ($store:expr) => {
            const LEN: usize = 4;
            let blocks: Vec<_> = (0..LEN).into_iter().map(create_block_raw).collect();
            let blocks = model::Shared::new(blocks);
            const LLEN: usize = 3;
            let links = model::Shared::new(["a", "b", "c"]);
            linearizable! {
                Implementation => let store = model::Shared::new($store),
                Read(usize)(i in 0..LEN) -> Option<Box<[u8]>> {
                    let (cid, _) = &blocks[i];
                    task::block_on(store.read(cid)).unwrap()
                },
                Write(usize)(i in 0..LEN) -> () {
                    let (cid, data) = &blocks[i];
                    task::block_on(store.write(cid, data.clone())).unwrap()
                },
                Flush(usize)(_ in 0..LEN) -> () {
                    task::block_on(store.flush()).unwrap()
                },
                Gc(usize)(_ in 0..LEN) -> () {
                    task::block_on(store.gc()).unwrap()
                },
                Pin(usize)(i in 0..LEN) -> () {
                    let (cid, _) = &blocks[i];
                    task::block_on(store.pin(cid)).unwrap()
                },
                Unpin(usize)(i in 0..LEN) -> () {
                    let (cid, _) = &blocks[i];
                    task::block_on(store.unpin(cid)).unwrap()
                },
                WriteLink((usize, usize))((i1, i2) in (0..LLEN, 0..LEN)) -> () {
                    let link = &links[i1];
                    let (cid, _) = &blocks[i2];
                    task::block_on(store.write_link(link, cid)).unwrap()
                },
                ReadLink(usize)(i in 0..LLEN) -> Option<Cid> {
                    let link = &links[i];
                    task::block_on(store.read_link(link)).unwrap()
                },
                RemoveLink(usize)(i in 0..LLEN) -> () {
                    let link = &links[i];
                    task::block_on(store.remove_link(link)).unwrap()
                }
            }
        };
    }

    #[test]
    fn mem_store_lin() {
        linearizable_store!(MemStore::default());
    }

    #[test]
    fn buf_store_lin() {
        linearizable_store!(BufStore::new(MemStore::default(), 16, 16));
    }
}
