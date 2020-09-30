//! Cache
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode, Encode};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::{Code, MultihashCode};
use crate::store::{Store, StoreParams};
use async_std::sync::Mutex;
use async_trait::async_trait;
use cached::stores::SizedCache;
use cached::Cached;

/// Cache for ipld blocks.
#[derive(Debug)]
pub struct IpldCache<S, C, T>
where
    S: Store,
    <S::Params as StoreParams>::Codecs: Into<C>,
    C: Codec + Into<<S::Params as StoreParams>::Codecs>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    store: S,
    codec: C,
    hash: u64,
    cache:
        Mutex<SizedCache<Cid<<<S::Params as StoreParams>::Hashes as MultihashCode>::AllocSize>, T>>,
}

impl<S: Default, C: Default, T> Default for IpldCache<S, C, T>
where
    S: Store,
    <S::Params as StoreParams>::Codecs: Into<C>,
    C: Codec + Into<<S::Params as StoreParams>::Codecs>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new(S::default(), C::default(), Code::Blake2s256.into(), 12)
    }
}

impl<S, C, T> IpldCache<S, C, T>
where
    S: Store,
    <S::Params as StoreParams>::Codecs: Into<C>,
    C: Codec + Into<<S::Params as StoreParams>::Codecs>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
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
    S: StoreParams,
    S::Codecs: Into<C>,
    C: Codec + Into<S::Codecs>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
{
    /// Returns a decoded block.
    async fn get(&self, cid: &Cid<<S::Hashes as MultihashCode>::AllocSize>) -> Result<T>;

    /// Encodes and inserts a block.
    async fn insert(&self, payload: T) -> Result<Cid<<S::Hashes as MultihashCode>::AllocSize>>;
}

#[async_trait]
impl<S, C, T> Cache<S::Params, C, T> for IpldCache<S, C, T>
where
    S: Store,
    <S::Params as StoreParams>::Codecs: Into<C>,
    C: Codec + Into<<S::Params as StoreParams>::Codecs>,
    T: Decode<C> + Encode<C> + Clone + Send + Sync,
    Ipld<<<S::Params as StoreParams>::Hashes as MultihashCode>::AllocSize>:
        Decode<<S::Params as StoreParams>::Codecs>,
{
    async fn get(
        &self,
        cid: &Cid<<<S::Params as StoreParams>::Hashes as MultihashCode>::AllocSize>,
    ) -> Result<T> {
        if let Some(value) = self.cache.lock().await.cache_get(cid).cloned() {
            return Ok(value);
        }
        let block = self.store.get(cid).await?;
        let value: T = block.decode::<C, _>()?;
        let (cid, _) = block.into_inner();
        self.cache.lock().await.cache_set(cid, value.clone());
        Ok(value)
    }

    async fn insert(
        &self,
        payload: T,
    ) -> Result<Cid<<<S::Params as StoreParams>::Hashes as MultihashCode>::AllocSize>> {
        let block = Block::encode(self.codec, self.hash, &payload)?;
        self.store.insert(&block).await?;
        let mut cache = self.cache.lock().await;
        cache.cache_set(block.cid().clone(), payload);
        Ok(block.cid().clone())
    }
}

/// Macro to derive cache trait for a struct.
#[macro_export]
macro_rules! derive_cache {
    ($struct:tt, $field:ident, $codec:ty, $type:ty) => {
        #[async_trait::async_trait]
        impl<S> $crate::cache::Cache<S::Params, $codec, $type> for $struct<S>
        where
            S: $crate::store::Store,
            <S::Params as $crate::store::StoreParams>::Codecs: From<$codec> + Into<$codec>,
            $crate::ipld::Ipld<
                <<S::Params as $crate::store::StoreParams>::Hashes as MultihashCode>::AllocSize,
            >: $crate::codec::Decode<<S::Params as $crate::store::StoreParams>::Codecs>,
        {
            async fn get(
                &self,
                cid: &$crate::cid::Cid<
                    <<S::Params as $crate::store::StoreParams>::Hashes as MultihashCode>::AllocSize,
                >,
            ) -> $crate::error::Result<$type> {
                self.$field.get(cid).await
            }

            async fn insert(
                &self,
                payload: $type,
            ) -> $crate::error::Result<
                $crate::cid::Cid<
                    <<S::Params as $crate::store::StoreParams>::Hashes as MultihashCode>::AllocSize,
                >,
            > {
                self.$field.insert(payload).await
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

    struct OffchainClient<S: Store>
    where
        <S::Params as StoreParams>::Codecs: Into<DagCborCodec> + From<DagCborCodec>,
        //DagCborCodec: From<<S::Params as StoreParams>::Codecs>,
    {
        store: S,
        number: IpldCache<S, DagCborCodec, u32>,
    }

    impl<S: Store> Deref for OffchainClient<S>
    where
        <S::Params as StoreParams>::Codecs: Into<DagCborCodec> + From<DagCborCodec>,
        //DagCborCodec: From<<S::Params as StoreParams>::Codecs>,
    {
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
            number: IpldCache::new(store, DagCborCodec, Code::Blake2b256.into(), 1),
        };
        let cid = client.insert(42).await.unwrap();
        let res = client.get(&cid).await.unwrap();
        assert_eq!(res, 42);
    }
}
