//! Cache
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode, References};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::store::{Store, StoreParams};
use async_lock::Mutex;
use async_trait::async_trait;
use cached::stores::SizedCache;
use cached::Cached;
use std::ops::Deref;

/// Cache for ipld blocks.
#[derive(Debug)]
pub struct IpldCache<S: Store, C, T> {
    store: S,
    codec: C,
    hash: <S::Params as StoreParams>::Hashes,
    cache: Mutex<SizedCache<Cid, T>>,
}

impl<S: Store, C, T> Deref for IpldCache<S, C, T> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl<S: Store + Default, C: Default, T> Default for IpldCache<S, C, T>
where
    <S::Params as StoreParams>::Hashes: Default,
{
    fn default() -> Self {
        Self::new(
            S::default(),
            C::default(),
            <S::Params as StoreParams>::Hashes::default(),
            12,
        )
    }
}

impl<S: Store, C, T> IpldCache<S, C, T> {
    /// Creates a new cache of size `size`.
    pub fn new(store: S, codec: C, hash: <S::Params as StoreParams>::Hashes, size: usize) -> Self {
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
pub trait Cache<S: Store, C, T> {
    /// Returns a decoded block.
    async fn get(&self, cid: &Cid, tmp: Option<&S::TempPin>) -> Result<T>;

    /// Encodes and inserts a block.
    async fn insert(&self, payload: T, tmp: Option<&S::TempPin>) -> Result<Cid>;
}

#[async_trait]
impl<S, C, T> Cache<S, C, T> for IpldCache<S, C, T>
where
    S: Store,
    <S::Params as StoreParams>::Codecs: Into<C>,
    C: Codec + Into<<S::Params as StoreParams>::Codecs>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
    Ipld: References<<S::Params as StoreParams>::Codecs>,
{
    async fn get(&self, cid: &Cid, tmp: Option<&S::TempPin>) -> Result<T> {
        if let Some(value) = self.cache.lock().await.cache_get(cid).cloned() {
            return Ok(value);
        }
        let block = self.store.get(cid, tmp).await?;
        let value: T = block.decode::<C, _>()?;
        let (cid, _) = block.into_inner();
        self.cache.lock().await.cache_set(cid, value.clone());
        Ok(value)
    }

    async fn insert(&self, payload: T, tmp: Option<&S::TempPin>) -> Result<Cid> {
        let block = Block::encode(self.codec, self.hash, &payload)?;
        self.store.insert(&block, tmp).await?;
        let mut cache = self.cache.lock().await;
        cache.cache_set(*block.cid(), payload);
        Ok(*block.cid())
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
            <S::Params as $crate::store::StoreParams>::Codecs: From<$codec> + Into<$codec>,
            $crate::ipld::Ipld:
                $crate::codec::References<<S::Params as $crate::store::StoreParams>::Codecs>,
        {
            async fn get(
                &self,
                cid: &$crate::cid::Cid,
                tmp: Option<&S::TempPin>,
            ) -> $crate::error::Result<$type> {
                self.$field.get(cid, tmp).await
            }

            async fn insert(
                &self,
                payload: $type,
                tmp: Option<&S::TempPin>,
            ) -> $crate::error::Result<$crate::cid::Cid> {
                self.$field.insert(payload, tmp).await
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor::DagCborCodec;
    use crate::mem::MemStore;
    use crate::multihash::Code;
    use crate::store::DefaultParams;
    use core::ops::Deref;

    struct OffchainClient<S: Store> {
        store: S,
        number: IpldCache<S, DagCborCodec, u32>,
    }

    impl<S: Store> Deref for OffchainClient<S> {
        type Target = S;

        fn deref(&self) -> &Self::Target {
            &self.store
        }
    }

    derive_cache!(OffchainClient, number, DagCborCodec, u32);

    #[async_std::test]
    async fn test_cache() {
        let store = MemStore::<DefaultParams>::default();
        let client = OffchainClient {
            store: store.clone(),
            number: IpldCache::new(store, DagCborCodec, Code::Blake3_256, 1),
        };
        let tmp = client.temp_pin().await.unwrap();
        let cid = client.insert(42, Some(&tmp)).await.unwrap();
        let res = client.get(&cid, Some(&tmp)).await.unwrap();
        assert_eq!(res, 42);
    }
}
