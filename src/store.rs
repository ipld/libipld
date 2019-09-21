//! Traits for implementing a block store.
use crate::block::{create_cbor_block, decode_cbor, decode_ipld, validate};
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::error::Result;
use crate::hash::Hash;
use crate::ipld::{Cid, Ipld};
use async_trait::async_trait;
use std::path::Path;

/// Implementable by ipld storage backends.
#[async_trait]
pub trait Store: Send + Sized {
    /// Returns the block with cid.
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>>;
    /// Writes the block with cid.
    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()>;

    /// Pin a block.
    async fn pin(&self, cid: &Cid) -> Result<()>;
    /// Unpin a block.
    async fn unpin(&self, cid: &Cid) -> Result<()>;
    /// Create an indirect user managed pin.
    async fn autopin(&self, cid: &Cid, _auto_path: &Path) -> Result<()> {
        self.pin(cid).await
    }

    /// Create a link to a block.
    async fn create_link(&self, label: &str, cid: &Cid) -> Result<()>;
    /// Read a link to a block.
    async fn read_link(&self, label: &str) -> Result<Option<Cid>>;
    /// Remove link to a block.
    async fn remove_link(&self, label: &str) -> Result<()>;
}

/// Implementable by ipld caches.
pub trait Cache: Send {
    /// Gets the block with cid from the cache.
    fn get(&self, cid: &Cid) -> Option<Box<[u8]>>;
    /// Puts the block with cid in to the cache.
    fn put(&self, cid: Cid, data: Box<[u8]>);
}

/// Generic block store with a parameterizable storage backend and cache.
pub struct BlockStore<TStore, TCache> {
    store: TStore,
    cache: TCache,
}

impl<TStore: Store, TCache: Cache> BlockStore<TStore, TCache> {
    /// Creates a new block store.
    pub fn new(store: TStore, cache: TCache) -> Self {
        Self { store, cache }
    }

    /// Reads the block with cid.
    #[inline]
    async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        if self.cache.get(cid).is_none() {
            if let Some(data) = self.store.read(cid).await? {
                validate(cid, &data)?;
                self.cache.put(cid.to_owned(), data);
            }
        }
        Ok(self.cache.get(cid))
    }

    /// Reads the block with cid and decodes it to ipld.
    pub async fn read_ipld(&self, cid: &Cid) -> Result<Option<Ipld>> {
        if let Some(data) = self.read(cid).await? {
            let ipld = decode_ipld(cid, &data).await?;
            return Ok(Some(ipld));
        }
        Ok(None)
    }

    /// Reads the block with cid and decodes it to cbor.
    pub async fn read_cbor<C: ReadCbor + Send>(&self, cid: &Cid) -> Result<Option<C>> {
        if let Some(data) = self.read(cid).await? {
            let cbor = decode_cbor::<C>(cid, &data).await?;
            return Ok(Some(cbor));
        }
        Ok(None)
    }

    /// Writes a block using the cbor codec.
    pub async fn write_cbor<H: Hash, C: WriteCbor>(&self, c: &C) -> Result<Cid> {
        let (cid, data) = create_cbor_block::<H, C>(c).await?;
        self.cache.put(cid.clone(), data.clone());
        self.store.write(&cid, data).await?;
        Ok(cid)
    }

    /// Flush to disk.
    pub async fn flush(&self) -> Result<()> {
        // TODO add a write buffer and gc the write buffer
        // before writing to disk.
        Ok(())
    }
}

pub mod mock {
    //! Utilities for testing
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::convert::TryFrom;
    use std::sync::Mutex;

    /// A memory backed store
    #[derive(Default)]
    pub struct MemStore {
        blocks: Mutex<HashMap<Box<[u8]>, Box<[u8]>>>,
        pins: Mutex<HashSet<Box<[u8]>>>,
        links: Mutex<HashMap<String, Box<[u8]>>>,
    }

    #[async_trait]
    impl Store for MemStore {
        async fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
            let key = cid.to_bytes().into_boxed_slice();
            let blocks = self.blocks.lock().unwrap();
            Ok(blocks.get(&key).cloned())
        }

        async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
            let key = cid.to_bytes().into_boxed_slice();
            let mut blocks = self.blocks.lock().unwrap();
            blocks.insert(key, data);
            Ok(())
        }

        async fn pin(&self, cid: &Cid) -> Result<()> {
            let key = cid.to_bytes().into_boxed_slice();
            let mut pins = self.pins.lock().unwrap();
            pins.insert(key);
            Ok(())
        }

        async fn unpin(&self, cid: &Cid) -> Result<()> {
            let key = cid.to_bytes().into_boxed_slice();
            let mut pins = self.pins.lock().unwrap();
            pins.remove(&key);
            Ok(())
        }

        async fn create_link(&self, link: &str, cid: &Cid) -> Result<()> {
            let key = cid.to_bytes().into_boxed_slice();
            let mut links = self.links.lock().unwrap();
            links.insert(link.to_string(), key);
            Ok(())
        }

        async fn read_link(&self, link: &str) -> Result<Option<Cid>> {
            let links = self.links.lock().unwrap();
            if let Some(bytes) = links.get(link) {
                let cid = Cid::try_from(bytes as &[u8])?;
                return Ok(Some(cid));
            }
            Ok(None)
        }

        async fn remove_link(&self, link: &str) -> Result<()> {
            let mut links = self.links.lock().unwrap();
            links.remove(link);
            Ok(())
        }
    }

    /// A memory backed cache
    #[derive(Default)]
    pub struct MemCache(Mutex<HashMap<Vec<u8>, Box<[u8]>>>);

    impl Cache for MemCache {
        fn get(&self, cid: &Cid) -> Option<Box<[u8]>> {
            self.0.lock().unwrap().get(&cid.to_bytes()).cloned()
        }

        fn put(&self, cid: Cid, data: Box<[u8]>) {
            let bytes = cid.to_bytes();
            self.0.lock().unwrap().insert(bytes.clone(), data);
        }
    }
}
