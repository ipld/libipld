//! Traits for implementing a block store.
use crate::block::{create_cbor_block, decode_cbor, decode_ipld, validate};
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::error::{format_err, Result};
use crate::hash::Hash;
use crate::ipld::{Cid, Ipld};
use async_trait::async_trait;

/// Implementable by ipld storage backends.
#[async_trait]
pub trait Store: Send + Sized {
    /// Returns the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    async fn read(&self, cid: &Cid) -> Result<Box<[u8]>>;
    /// Writes the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()>;
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
        Self {
            store,
            cache,
        }
    }

    /// Reads the block with cid.
    #[inline]
    async fn read(&self, cid: &Cid) -> Result<Box<[u8]>> {
        if self.cache.get(cid).is_none() {
            let data = self.store.read(cid).await?;
            validate(cid, &data)?;
            self.cache.put(cid.to_owned(), data);
        }
        Ok(self.cache.get(cid).expect("in cache"))
    }

    /// Reads the block with cid and decodes it to ipld.
    pub async fn read_ipld(&self, cid: &Cid) -> Result<Ipld> {
        let data = self.read(cid).await?;
        let ipld = decode_ipld(cid, &data).await?;
        Ok(ipld)
    }

    /// Reads the block with cid and decodes it to cbor.
    pub async fn read_cbor<C: ReadCbor + Send>(&self, cid: &Cid) -> Result<C> {
        let data = self.read(cid).await?;
        let cbor = decode_cbor::<C>(cid, &data).await?;
        Ok(cbor)
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
    use multibase::Base;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// A memory backed store
    #[derive(Default)]
    pub struct MemStore(Arc<Mutex<HashMap<String, Box<[u8]>>>>);

    impl MemStore {
        #[inline]
        fn key(&self, cid: &Cid) -> String {
            multibase::encode(Base::Base64UpperNoPad, cid.to_bytes())
        }
    }

    #[async_trait]
    impl Store for MemStore {
        async fn read(&self, cid: &Cid) -> Result<Box<[u8]>> {
            let key = self.key(cid);
            if let Some(data) = self.0.lock().unwrap().get(&key) {
                Ok(data.to_owned())
            } else {
                Err(format_err!("Block not found"))
            }
        }

        async fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
            let key = self.key(cid);
            self.0.lock().unwrap().insert(key, data);
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
