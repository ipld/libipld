//! Store traits.
//!
//! ## Aliases
//! An alias is a named root of a dag. When a root is aliased, none of the leaves of the dag
//! pointed to by the root will be collected by gc. However, a root being aliased does not
//! mean that the dag must be complete.
//!
//! ## Temporary pin
//! A temporary pin is an unnamed set of roots of a dag, that is just for the purpose of protecting
//! blocks from gc while a large tree is constructed. While an alias maps a single name to a
//! single root, a temporary alias can be assigned to an arbitrary number of blocks before the
//! dag is finished.
//!
//! ## Garbage collection (GC)
//! GC refers to the process of removing unaliased blocks. When it runs is implementation defined.
//! However it is intended to run only when the configured size is exceeded at when it will start
//! incrementally deleting unaliased blocks until the size target is no longer exceeded. It is
//! implementation defined in which order unaliased blocks get removed.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::{Codec, Decode};
use crate::error::Result;
use crate::ipld::Ipld;
use crate::multihash::MultihashDigest;
use crate::path::DagPath;
use async_trait::async_trait;

/// The store parameters.
pub trait StoreParams: std::fmt::Debug + Clone + Send + Sync + Unpin + 'static {
    /// The multihash type of the store.
    type Hashes: MultihashDigest<64>;
    /// The codec type of the store.
    type Codecs: Codec;
    /// The maximum block size supported by the store.
    const MAX_BLOCK_SIZE: usize;
}

/// Default store parameters.
#[derive(Clone, Debug, Default)]
pub struct DefaultParams;

impl StoreParams for DefaultParams {
    const MAX_BLOCK_SIZE: usize = 1_048_576;
    type Codecs = crate::IpldCodec;
    type Hashes = crate::multihash::Code;
}

/// Implementable by ipld stores. An ipld store behaves like a cache. It will keep blocks
/// until the cache is full after which it evicts blocks based on an eviction policy. If
/// a block is aliased (recursive named pin), it and it's recursive references will not
/// be evicted or counted towards the cache size.
#[async_trait]
pub trait Store: Clone + Send + Sync {
    /// Store parameters.
    type Params: StoreParams;
    /// Temp pin.
    type TempPin: Clone + Send + Sync;

    /// Creates a new temporary pin.
    fn create_temp_pin(&self) -> Result<Self::TempPin>;

    /// Adds a block to a temp pin.
    fn temp_pin(&self, tmp: &Self::TempPin, cid: &Cid) -> Result<()>;

    /// Returns true if the store contains the block.
    fn contains(&self, cid: &Cid) -> Result<bool>;

    /// Returns a block from the store. If the block wasn't found it returns a `BlockNotFound`
    /// error.
    fn get(&self, cid: &Cid) -> Result<Block<Self::Params>>;

    /// Inserts a block into the store and publishes the block on the network.
    fn insert(&self, block: &Block<Self::Params>) -> Result<()>;

    /// Creates an alias for a `Cid`.
    fn alias<T: AsRef<[u8]> + Send + Sync>(&self, alias: T, cid: Option<&Cid>) -> Result<()>;

    /// Resolves an alias for a `Cid`.
    fn resolve<T: AsRef<[u8]> + Send + Sync>(&self, alias: T) -> Result<Option<Cid>>;

    /// Returns all the aliases that are keeping the block around.
    fn reverse_alias(&self, cid: &Cid) -> Result<Option<Vec<Vec<u8>>>>;

    /// Flushes the store.
    async fn flush(&self) -> Result<()>;

    /// Returns a block from the store. If the store supports networking and the block is not
    /// in the store it fetches it from the network and inserts it into the store. Dropping the
    /// future cancels the request.
    ///
    /// If the block wasn't found it returns a `BlockNotFound` error.
    async fn fetch(&self, cid: &Cid) -> Result<Block<Self::Params>>;

    /// Fetches all missing blocks recursively from the network. If a block isn't found it
    /// returns a `BlockNotFound` error.
    async fn sync(&self, cid: &Cid) -> Result<()>;

    /// Resolves a path recursively and returns the ipld.
    async fn query(&self, path: &DagPath<'_>) -> Result<Ipld>
    where
        Ipld: Decode<<Self::Params as StoreParams>::Codecs>,
    {
        let mut ipld = self.fetch(path.root()).await?.ipld()?;
        for segment in path.path().iter() {
            ipld = ipld.take(segment)?;
            if let Ipld::Link(cid) = ipld {
                ipld = self.fetch(&cid).await?.ipld()?;
            }
        }
        Ok(ipld)
    }
}

/// Creates a static alias concatenating the module path with an identifier.
#[macro_export]
macro_rules! alias {
    ($name:ident) => {
        concat!(module_path!(), "::", stringify!($name))
    };
}

/// Creates a dynamic alias by appending a id.
pub fn dyn_alias(alias: &'static str, id: u64) -> String {
    let mut alias = alias.to_string();
    alias.push_str("::");
    alias.push_str(&id.to_string());
    alias
}

#[cfg(test)]
mod tests {
    use super::*;
    use lurk_ipld_macro::ipld;

    mod aliases {
        pub const CHAIN_ALIAS: &str = alias!(CHAIN_ALIAS);
    }

    #[test]
    fn test_alias() {
        assert_eq!(alias!(test_alias), "lurk_ipld::store::tests::test_alias");
        assert_eq!(
            aliases::CHAIN_ALIAS,
            "lurk_ipld::store::tests::aliases::CHAIN_ALIAS"
        );
        assert_eq!(
            dyn_alias(aliases::CHAIN_ALIAS, 3).as_str(),
            "lurk_ipld::store::tests::aliases::CHAIN_ALIAS::3"
        );
    }

    #[async_std::test]
    async fn test_query() -> Result<()> {
        use crate::cbor::DagCborCodec;
        use crate::mem::MemStore;
        use crate::multihash::Code;

        let store = MemStore::<DefaultParams>::default();
        let leaf = ipld!({"name": "John Doe"});
        let leaf_block = Block::encode(DagCborCodec, Code::Blake3_256, &leaf).unwrap();
        let root = ipld!({ "list": [leaf_block.cid()] });
        store.insert(&leaf_block).unwrap();
        let root_block = Block::encode(DagCborCodec, Code::Blake3_256, &root).unwrap();
        store.insert(&root_block).unwrap();
        let path = DagPath::new(root_block.cid(), "list/0/name");
        assert_eq!(
            store.query(&path).await.unwrap(),
            Ipld::String("John Doe".to_string())
        );
        Ok(())
    }
}
