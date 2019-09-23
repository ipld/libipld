//! Traits for implementing a block store.
use crate::block::{create_cbor_block, decode_cbor, decode_ipld, validate};
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::error::Result;
use crate::hash::Hash;
use crate::ipld::{Cid, Ipld};
use async_std::sync::RwLock;
use async_trait::async_trait;
use core::hash::{BuildHasher, Hasher};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Implementable by ipld storage backends.
#[async_trait]
pub trait Store: Send + Sync + Sized {
    /// Returns the block with cid.
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>>;
    /// Writes the block with cid.
    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()>;
    /// Flushes the write buffer.
    async fn flush(&self) -> Result<()>;

    /// Pin a block.
    async fn pin(&self, cid: &Cid) -> Result<()>;
    /// Unpin a block.
    async fn unpin(&self, cid: &Cid) -> Result<()>;
    /// Create an indirect user managed pin.
    async fn autopin(&self, cid: &Cid, _auto_path: &Path) -> Result<()>;

    /// Write a link to a block.
    async fn write_link(&self, label: &str, cid: &Cid) -> Result<()>;
    /// Read a link to a block.
    async fn read_link(&self, label: &str) -> Result<Option<Cid>>;
    /// Remove link to a block.
    async fn remove_link(&self, label: &str) -> Result<()>;
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
