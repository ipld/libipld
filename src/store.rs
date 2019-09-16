//! Traits for implementing a block store.
use crate::codec::cbor::{ReadCbor, WriteCbor};
use crate::codec::decode;
use crate::error::{format_err, Result};
use crate::hash::{digest, Hash};
use crate::ipld::{Cid, Ipld};
use async_trait::async_trait;
use cid::Codec;

/// Implementable by ipld storage backends.
#[async_trait]
pub trait Store: Default {
    /// Returns the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    async fn read(&self, cid: &Cid) -> Result<Box<[u8]>>;
    /// Writes the block with cid. It is marked unsafe because the caller must
    ///  ensure that the hash matches the data.
    fn write(&mut self, cid: &Cid, data: &Box<[u8]>) -> Result<()>;
    /// Deletes the block that match the cid.
    fn delete(&mut self, cid: &Cid) -> Result<()>;
    /// Flushes changes to disk.
    async fn flush(&mut self) -> Result<()>;
}

/// Implementable by ipld caches.
pub trait Cache {
    /// Create a new cache of size cap.
    fn new(cap: usize) -> Self;
    /// Gets the block with cid from the cache.
    fn get(&mut self, cid: &Cid) -> Option<&Box<[u8]>>;
    /// Puts the block with cid in to the cache.
    fn put(&mut self, cid: &Cid, data: Box<[u8]>);
    /// Evicts the block with cid from the cache.
    fn evict(&mut self, cid: &Cid);
}

/// Generic block store with a parameterizable storage backend and cache.
pub struct BlockStore<TStore, TCache> {
    store: TStore,
    cache: TCache,
}

impl<TStore: Store, TCache: Cache> BlockStore<TStore, TCache> {
    /// Creates a new block store.
    pub fn new(cache_size: usize) -> Self {
        Self {
            store: Default::default(),
            cache: TCache::new(cache_size),
        }
    }

    /// Reads the block with cid.
    #[inline]
    pub async fn read(&mut self, cid: &Cid) -> Result<&Box<[u8]>> {
        if self.cache.get(cid).is_none() {
            let data = self.store.read(cid).await?;
            let hash = digest(cid.hash().code(), &data);
            if cid.hash() != hash.as_ref() {
                return Err(format_err!("Invalid hash"));
            }
            self.cache.put(cid, data);
        }
        Ok(self.cache.get(cid).expect("in cache"))
    }

    /// Reads the block with cid and decodes it to ipld.
    pub async fn read_ipld(&mut self, cid: &Cid) -> Result<Ipld> {
        let data = self.read(cid).await?;
        decode(cid.codec(), &data)
    }

    /// Reads the block with cid and decodes it to cbor.
    pub async fn read_cbor<C: ReadCbor>(&mut self, cid: &Cid) -> Result<C> {
        if cid.codec() != cid::Codec::DagCBOR {
            return Err(format_err!("Not cbor codec"));
        }
        let data = self.read(cid).await?;
        let mut data_ref: &[u8] = &data;
        C::read_cbor(&mut data_ref)
    }

    /// Writes a block using the cbor codec.
    pub fn write_cbor<H: Hash, C: WriteCbor>(&mut self, c: &C) -> Result<Cid> {
        let mut data = Vec::new();
        c.write_cbor(&mut data)?;
        let hash = H::digest(&data);
        let cid = Cid::new_v1(Codec::DagCBOR, hash);
        self.cache.put(&cid, data.into_boxed_slice());
        let data = self.cache.get(&cid).expect("in cache");
        self.store.write(&cid, data)?;
        Ok(cid)
    }

    /// Deletes the block with cid.
    pub fn delete(&mut self, cid: &Cid) -> Result<()> {
        self.cache.evict(cid);
        self.store.delete(cid)?;
        Ok(())
    }

    /// Flushes changes to disk.
    pub async fn flush(&mut self) -> Result<()> {
        self.store.flush().await
    }
}

pub mod mock {
    //! Utilities for testing
    use super::*;
    use multibase::Base;
    use std::collections::HashMap;

    /// A memory backed store
    #[derive(Default)]
    pub struct MemStore(HashMap<String, Box<[u8]>>);

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
            if let Some(data) = self.0.get(&key) {
                Ok(data.to_owned())
            } else {
                Err(format_err!("Block not found"))
            }
        }

        fn write(&mut self, cid: &Cid, data: &Box<[u8]>) -> Result<()> {
            let key = self.key(cid);
            self.0.insert(key, data.to_owned());
            Ok(())
        }

        fn delete(&mut self, cid: &Cid) -> Result<()> {
            let key = self.key(cid);
            self.0.remove(&key);
            Ok(())
        }

        async fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    /// A memory backed cache
    pub struct MemCache(HashMap<Vec<u8>, Box<[u8]>>);

    impl Cache for MemCache {
        fn new(_cap: usize) -> Self {
            Self(Default::default())
        }

        fn get(&mut self, cid: &Cid) -> Option<&Box<[u8]>> {
            self.0.get(&cid.to_bytes())
        }

        fn put(&mut self, cid: &Cid, data: Box<[u8]>) {
            let bytes = cid.to_bytes();
            self.0.insert(bytes.clone(), data);
        }

        fn evict(&mut self, cid: &Cid) {
            self.0.remove(&cid.to_bytes());
        }
    }
}
