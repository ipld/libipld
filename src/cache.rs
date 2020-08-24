//! Cache
use crate::block::{Block, Visibility};
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::Result;
use crate::multihash::BLAKE2B_256;
use crate::store::{ReadonlyStore, Store};
use async_std::sync::Mutex;
use async_trait::async_trait;
use cached::stores::SizedCache;
use cached::Cached;

/// Cache config.
pub struct CacheConfig<S, C> {
    /// Backing store.
    pub store: S,
    /// The codec used to encode blocks.
    pub codec: C,
    /// The hash used to encode blocks.
    pub hash: u64,
    /// The visibility of encoded blocks.
    pub visibility: Visibility,
    /// The cache size.
    pub size: usize,
    /// The default batch capacity when unspecified.
    pub batch_capacity: usize,
}

impl<S, C> CacheConfig<S, C> {
    /// Creates a new config with reasonable defaults.
    pub fn new(store: S, codec: C) -> Self {
        Self {
            store,
            codec,
            hash: BLAKE2B_256,
            visibility: Visibility::Public,
            size: 4,
            batch_capacity: 4,
        }
    }
}

/// Cache for ipld blocks.
pub struct IpldCache<S, C, T> {
    config: CacheConfig<S, C>,
    cache: Mutex<SizedCache<Cid, T>>,
}

impl<S, C, T> IpldCache<S, C, T> {
    /// Creates a new cache of size `size`.
    pub fn new(config: CacheConfig<S, C>) -> Self {
        let cache = Mutex::new(SizedCache::with_size(config.size));
        Self { config, cache }
    }
}

/// Readonly cache trait.
#[async_trait]
pub trait ReadonlyCache<S, C, T>
where
    S: ReadonlyStore,
    S::Codec: Into<C>,
    C: Codec,
    T: Decode<C> + Clone + Send + Sync,
{
    /// Returns a decoded block.
    async fn get(&self, cid: &Cid) -> Result<T>;
}

#[async_trait]
impl<S, C, T> ReadonlyCache<S, C, T> for IpldCache<S, C, T>
where
    S: ReadonlyStore,
    S::Codec: Into<C>,
    C: Codec,
    T: Decode<C> + Clone + Send + Sync,
{
    async fn get(&self, cid: &Cid) -> Result<T> {
        if let Some(value) = self.cache.lock().await.cache_get(cid).cloned() {
            return Ok(value);
        }
        let block = self.config.store.get(cid.clone()).await?;
        let value: T = block.decode::<C, _>()?;
        self.cache.lock().await.cache_set(block.cid, value.clone());
        Ok(value)
    }
}

/// Typed batch.
pub struct Batch<S: Store, C, T> {
    codec: C,
    hash: u64,
    vis: Visibility,
    cache: Vec<(Cid, T)>,
    batch: Vec<Block<S::Codec, S::Multihash>>,
}

impl<S, C, T> Batch<S, C, T>
where
    S: Store,
    C: Codec + Into<S::Codec>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    /// Creates a new batch with capacity.
    fn new(codec: C, hash: u64, vis: Visibility, capacity: usize) -> Self {
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

/// Cache trait.
#[async_trait]
pub trait Cache<S, C, T>: ReadonlyCache<S, C, T>
where
    S: Store,
    S::Codec: Into<C>,
    C: Codec + Into<S::Codec>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    /// Creates a typed batch.
    fn create_batch(&self) -> Batch<S, C, T>;

    /// Creates a typed batch.
    fn create_batch_with_capacity(&self, capacity: usize) -> Batch<S, C, T>;

    /// Inserts a batch into the store.
    async fn insert_batch(&self, batch: Batch<S, C, T>) -> Result<Cid>;

    /// Encodes and inserts a block.
    async fn insert(&self, value: T) -> Result<Cid>;

    /// Flushes all buffers.
    async fn flush(&self) -> Result<()>;

    /// Unpins a block.
    async fn unpin(&self, cid: &Cid) -> Result<()>;
}

#[async_trait]
impl<S, C, T> Cache<S, C, T> for IpldCache<S, C, T>
where
    S: Store,
    S::Codec: Into<C>,
    C: Codec + Into<S::Codec>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    fn create_batch(&self) -> Batch<S, C, T> {
        self.create_batch_with_capacity(self.config.batch_capacity)
    }

    fn create_batch_with_capacity(&self, capacity: usize) -> Batch<S, C, T> {
        Batch::new(
            self.config.codec,
            self.config.hash,
            self.config.visibility,
            capacity,
        )
    }

    async fn insert_batch(&self, batch: Batch<S, C, T>) -> Result<Cid> {
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

/// Macro to derive cache trait for a struct.
#[macro_export]
macro_rules! derive_cache {
    ($struct:tt, $field:ident, $codec:ty, $type:ty) => {
        #[async_trait::async_trait]
        impl<S> $crate::cache::ReadonlyCache<S, $codec, $type> for $struct<S>
        where
            S: $crate::store::ReadonlyStore,
            S::Codec: Into<$codec>,
        {
            async fn get(&self, cid: &$crate::cid::Cid) -> $crate::error::Result<$type> {
                self.$field.get(cid).await
            }
        }

        #[async_trait::async_trait]
        impl<S> $crate::cache::Cache<S, $codec, $type> for $struct<S>
        where
            S: $crate::store::Store,
            S::Codec: From<$codec> + Into<$codec>,
        {
            fn create_batch(&self) -> $crate::cache::Batch<S, $codec, $type> {
                self.$field.create_batch()
            }

            fn create_batch_with_capacity(
                &self,
                capacity: usize,
            ) -> $crate::cache::Batch<S, $codec, $type> {
                self.$field.create_batch_with_capacity(capacity)
            }

            async fn insert_batch(
                &self,
                batch: $crate::cache::Batch<S, $codec, $type>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor::DagCborCodec;
    use crate::codec_impl::Multicodec;
    use crate::mem::MemStore;
    use crate::multihash::Multihash;

    struct OffchainClient<S> {
        number: IpldCache<S, DagCborCodec, u32>,
    }

    derive_cache!(OffchainClient, number, DagCborCodec, u32);

    #[async_std::test]
    async fn test_cache() {
        let store = MemStore::<Multicodec, Multihash>::new();
        let config = CacheConfig::new(store, DagCborCodec);
        let client = OffchainClient {
            number: IpldCache::new(config),
        };
        let cid = client.insert(42).await.unwrap();
        let res = client.get(&cid).await.unwrap();
        assert_eq!(res, 42);
    }
}
