//! Cache
use crate::block::{Block, Visibility};
use crate::cid::{Cid, DAG_CBOR};
use crate::codec::{Codec, Decode, Encode};
use crate::error::Result;
use crate::multihash::{MultihashDigest, BLAKE2B_256};
use crate::store::{ReadonlyStore, Store};
use async_std::sync::Mutex;
use async_trait::async_trait;
use cached::stores::SizedCache;
use cached::Cached;

/// Cache config.
pub struct CacheConfig<S> {
    /// Backing store.
    pub store: S,
    /// The codec used to encode blocks.
    pub codec: u64,
    /// The hash used to encode blocks.
    pub hash: u64,
    /// The visibility of encoded blocks.
    pub visibility: Visibility,
    /// The cache size.
    pub size: usize,
    /// The default batch capacity when unspecified.
    pub batch_capacity: usize,
}

impl<S> CacheConfig<S> {
    /// Creates a new config with reasonable defaults.
    pub fn new(store: S) -> Self {
        Self {
            store,
            codec: DAG_CBOR,
            hash: BLAKE2B_256,
            visibility: Visibility::Public,
            size: 4,
            batch_capacity: 4,
        }
    }
}

/// Cache for ipld blocks.
pub struct IpldCache<S, T> {
    config: CacheConfig<S>,
    cache: Mutex<SizedCache<Cid, T>>,
}

impl<S, T> IpldCache<S, T> {
    /// Creates a new cache of size `size`.
    pub fn new(config: CacheConfig<S>) -> Self {
        let cache = Mutex::new(SizedCache::with_size(config.size));
        Self { config, cache }
    }
}

/// Readonly cache trait.
#[async_trait]
pub trait ReadonlyCache<S, T>
where
    S: ReadonlyStore,
    T: Decode<S::Codec> + Clone + Send + Sync,
{
    /// Returns a decoded block.
    async fn get(&self, cid: &Cid) -> Result<T>;
}

#[async_trait]
impl<S, T> ReadonlyCache<S, T> for IpldCache<S, T>
where
    S: ReadonlyStore,
    T: Decode<S::Codec> + Clone + Send + Sync,
{
    async fn get(&self, cid: &Cid) -> Result<T> {
        if let Some(value) = self.cache.lock().await.cache_get(cid).cloned() {
            return Ok(value);
        }
        let block = self.config.store.get(cid.clone()).await?;
        let value: T = block.decode()?;
        self.cache.lock().await.cache_set(block.cid, value.clone());
        Ok(value)
    }
}

/// Cache trait.
#[async_trait]
pub trait Cache<S, T>: ReadonlyCache<S, T>
where
    S: Store,
    T: Decode<S::Codec> + Encode<S::Codec> + Clone + Send + Sync,
{
    /// Creates a typed batch.
    fn create_batch(&self) -> Batch<S::Codec, S::Multihash, T>;

    /// Creates a typed batch.
    fn create_batch_with_capacity(&self, capacity: usize) -> Batch<S::Codec, S::Multihash, T>;

    /// Inserts a batch into the store.
    async fn insert_batch(&self, batch: Batch<S::Codec, S::Multihash, T>) -> Result<Cid>;

    /// Encodes and inserts a block.
    async fn insert(&self, value: T) -> Result<Cid>;

    /// Flushes all buffers.
    async fn flush(&self) -> Result<()>;

    /// Unpins a block.
    async fn unpin(&self, cid: &Cid) -> Result<()>;
}

#[async_trait]
impl<S, T> Cache<S, T> for IpldCache<S, T>
where
    S: Store,
    T: Decode<S::Codec> + Encode<S::Codec> + Clone + Send + Sync,
{
    fn create_batch(&self) -> Batch<S::Codec, S::Multihash, T> {
        self.create_batch_with_capacity(self.config.batch_capacity)
    }

    fn create_batch_with_capacity(&self, capacity: usize) -> Batch<S::Codec, S::Multihash, T> {
        Batch::new(
            self.config.codec,
            self.config.hash,
            self.config.visibility,
            capacity,
        )
    }

    async fn insert_batch(&self, batch: Batch<S::Codec, S::Multihash, T>) -> Result<Cid> {
        let cid = self.config.store.insert_batch(&batch.batch).await?;
        let mut cache = self.cache.lock().await;
        for (cid, value) in batch.cache {
            cache.cache_set(cid, value);
        }
        Ok(cid)
    }

    async fn insert(&self, value: T) -> Result<Cid> {
        let mut block = Block::encode(self.config.codec, self.config.hash, &value)?;
        block.set_visibility(self.config.visibility);
        self.config.store.insert(&block).await?;
        self.cache.lock().await.cache_set(block.cid.clone(), value);
        Ok(block.cid)
    }

    async fn flush(&self) -> Result<()> {
        self.config.store.flush().await
    }

    async fn unpin(&self, cid: &Cid) -> Result<()> {
        self.config.store.unpin(cid).await
    }
}

/// Typed batch.
pub struct Batch<C, M, T> {
    codec: u64,
    hash: u64,
    vis: Visibility,
    cache: Vec<(Cid, T)>,
    batch: Vec<Block<C, M>>,
}

impl<C, M, T> Batch<C, M, T>
where
    C: Codec,
    M: MultihashDigest,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    /// Creates a new batch with capacity.
    fn new(codec: u64, hash: u64, vis: Visibility, capacity: usize) -> Self {
        Self {
            codec,
            hash,
            vis,
            cache: Vec::with_capacity(capacity),
            batch: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a value into the batch.
    pub fn insert(&mut self, value: T) -> Result<Cid> {
        let mut block = Block::encode(self.codec, self.hash, &value)?;
        block.set_visibility(self.vis);
        let cid = block.cid.clone();
        self.batch.push(block);
        self.cache.push((cid.clone(), value));
        Ok(cid)
    }
}

/// Macro to derive cache trait for a struct.
#[macro_export]
macro_rules! derive_cache {
    ($struct:tt, $field:ident, $type:ty) => {
        #[async_trait::async_trait]
        impl<S> $crate::cache::ReadonlyCache<S, $type> for $struct<S>
        where
            S: $crate::store::ReadonlyStore,
        {
            async fn get(&self, cid: &$crate::cid::Cid) -> $crate::error::Result<$type> {
                self.$field.get(cid).await
            }
        }

        #[async_trait::async_trait]
        impl<S> $crate::cache::Cache<S, $type> for $struct<S>
        where
            S: $crate::store::Store,
        {
            fn create_batch(&self) -> $crate::cache::Batch<S::Codec, S::Multihash, $type> {
                self.$field.create_batch()
            }

            fn create_batch_with_capacity(
                &self,
                capacity: usize,
            ) -> $crate::cache::Batch<S::Codec, S::Multihash, $type> {
                self.$field.create_batch_with_capacity(capacity)
            }

            async fn insert_batch(
                &self,
                batch: $crate::cache::Batch<S::Codec, S::Multihash, $type>,
            ) -> $crate::error::Result<$crate::cid::Cid> {
                self.$field.insert_batch(batch).await
            }

            async fn insert(&self, value: $type) -> $crate::error::Result<$crate::cid::Cid> {
                self.$field.insert(value).await
            }

            async fn flush(&self) -> $crate::error::Result<()> {
                self.$field.flush().await
            }

            async fn unpin(&self, cid: &$crate::cid::Cid) -> $crate::error::Result<()> {
                self.$field.unpin(cid).await
            }
        }
    };
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::MemStore;

    struct OffchainClient<S> {
        number: IpldCache<S, u32>,
    }

    derive_cache!(OffchainClient, number, u32);

    #[async_std::test]
    async fn test_cache() {
        let store = MemStore::default();
        let config = CacheConfig::new(store);
        let client = OffchainClient {
            number: IpldCache::new(config),
        };
        let cid = client.insert(42).await.unwrap();
        let res = client.get(&cid).await.unwrap();
        assert_eq!(res, 42);
    }
}
*/
