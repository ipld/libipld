//! Cache
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::Result;
use crate::store::{Store, Transaction as RawTransaction};
use async_std::sync::Mutex;
use async_trait::async_trait;
use cached::stores::SizedCache;
use cached::Cached;

/// Typed transaction.
pub struct Transaction<S: Store, C, T> {
    codec: C,
    hash: u64,
    tx: RawTransaction<S::Codec, S::Multihash>,
    cache: Vec<(Cid, T)>,
}

impl<S, C, T> Transaction<S, C, T>
where
    S: Store,
    C: Codec + Into<S::Codec>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    /// Creates a new transaction.
    pub fn new(codec: C, hash: u64) -> Self {
        Self {
            codec,
            hash,
            tx: RawTransaction::new(),
            cache: Vec::new(),
        }
    }

    /// Creates a new batch with capacity.
    pub fn with_capacity(codec: C, hash: u64, capacity: usize) -> Self {
        Self {
            codec,
            hash,
            tx: RawTransaction::with_capacity(capacity),
            cache: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a value into the batch.
    pub fn insert(&mut self, value: T) -> Result<Cid> {
        let cid = self.tx.encode(self.codec, self.hash, &value)?;
        self.cache.push((cid.clone(), value));
        Ok(cid)
    }

    /// Pins a block.
    pub fn pin(&mut self, cid: Cid) {
        self.tx.pin(cid);
    }

    /// Pins a block.
    pub fn unpin(&mut self, cid: Cid) {
        self.tx.unpin(cid);
    }

    /// Updates a block.
    pub fn update(&mut self, old: Option<Cid>, new: Cid) {
        self.tx.update(old, new);
    }
}

/// Cache for ipld blocks.
pub struct IpldCache<S, C, T> {
    store: S,
    codec: C,
    hash: u64,
    cache: Mutex<SizedCache<Cid, T>>,
}

impl<S, C, T> IpldCache<S, C, T> {
    /// Creates a new cache of size `size`.
    pub fn new(store: S, codec: C, hash: u64, size: usize) -> Self {
        let cache = Mutex::new(SizedCache::with_size(size));
        Self {
            store,
            codec,
            hash,
            cache,
        }
    }
}

/// Cache trait.
#[async_trait]
pub trait Cache<S, C, T>
where
    S: Store,
    S::Codec: Into<C>,
    C: Codec + Into<S::Codec>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    /// Creates a transaction.
    fn transaction(&self) -> Transaction<S, C, T>;

    /// Creates a transaction with capacity.
    fn transaction_with_capacity(&self, capacity: usize) -> Transaction<S, C, T>;

    /// Returns a decoded block.
    async fn get(&self, cid: &Cid) -> Result<T>;

    /// Commits a transaction.
    async fn commit(&self, tx: Transaction<S, C, T>) -> Result<()>;
}

#[async_trait]
impl<S, C, T> Cache<S, C, T> for IpldCache<S, C, T>
where
    S: Store,
    S::Codec: Into<C>,
    C: Codec + Into<S::Codec>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    fn transaction(&self) -> Transaction<S, C, T> {
        Transaction::new(self.codec, self.hash)
    }

    fn transaction_with_capacity(&self, capacity: usize) -> Transaction<S, C, T> {
        Transaction::with_capacity(self.codec, self.hash, capacity)
    }

    async fn get(&self, cid: &Cid) -> Result<T> {
        if let Some(value) = self.cache.lock().await.cache_get(cid).cloned() {
            return Ok(value);
        }
        let block = self.store.get(cid.clone()).await?;
        let value: T = block.decode::<C, _>()?;
        let (cid, _) = block.destruct();
        self.cache.lock().await.cache_set(cid, value.clone());
        Ok(value)
    }

    async fn commit(&self, transaction: Transaction<S, C, T>) -> Result<()> {
        self.store.commit(transaction.tx).await?;
        let mut cache = self.cache.lock().await;
        for (cid, value) in transaction.cache {
            cache.cache_set(cid, value);
        }
        Ok(())
    }
}

/// Macro to derive cache trait for a struct.
#[macro_export]
macro_rules! derive_cache {
    ($struct:tt, $field:ident, $codec:ty, $type:ty) => {
        #[async_trait::async_trait]
        impl<S> $crate::cache::Cache<S, $codec, $type> for $struct<S>
        where
            S: $crate::store::Store,
            S::Codec: From<$codec> + Into<$codec>,
        {
            fn transaction(&self) -> $crate::cache::Transaction<S, $codec, $type> {
                self.$field.transaction()
            }

            fn transaction_with_capacity(
                &self,
                capacity: usize,
            ) -> $crate::cache::Transaction<S, $codec, $type> {
                self.$field.transaction_with_capacity()
            }

            async fn get(&self, cid: &$crate::cid::Cid) -> $crate::error::Result<$type> {
                self.$field.get(cid).await
            }

            async fn commit(
                &self,
                tx: $crate::cache::Transaction<S, $codec, $type>,
            ) -> $crate::error::Result<()> {
                self.$field.commit(tx).await
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
        let store = MemStore::<Multicodec, Multihash>::default();
        let config = CacheConfig::new(store, DagCborCodec);
        let client = OffchainClient {
            number: IpldCache::new(config),
        };
        let cid = client.insert(42).await.unwrap();
        let res = client.get(&cid).await.unwrap();
        assert_eq!(res, 42);
        client.unpin(&cid).await.unwrap();
    }
}
