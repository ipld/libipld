//! Reference store implementation.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::Decode;
use crate::error::{BlockNotFound, Result};
use crate::ipld::Ipld;
use crate::store::{Store, StoreParams};
use async_trait::async_trait;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

type Id = u64;
type Atime = u64;

struct BlockStore<S: StoreParams> {
    next_id: Id,
    blocks: HashMap<Id, Block<S>>,
    lookup: HashMap<Cid, Id>,
}

impl<S: StoreParams> BlockStore<S> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            blocks: Default::default(),
            lookup: Default::default(),
        }
    }

    pub fn get<'a>(&'a self, id: Id) -> Option<&'a Block<S>> {
        self.blocks.get(&id)
    }

    pub fn lookup(&self, cid: &Cid) -> Option<Id> {
        self.lookup.get(cid).cloned()
    }

    pub fn insert(&mut self, block: Block<S>) -> Option<Id> {
        if self.lookup.get(block.cid()).is_some() {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.lookup.insert(*block.cid(), id);
        self.blocks.insert(id, block);
        Some(id)
    }

    pub fn remove(&mut self, id: Id) {
        if let Some(block) = self.blocks.remove(&id) {
            self.lookup.remove(block.cid());
        }
    }

    pub fn references(&self, id: Id) -> Result<HashSet<Id>>
    where
        Ipld: Decode<S::Codecs>,
    {
        let mut refs = HashSet::default();
        let mut todo = Vec::new();
        todo.push(id);
        while let Some(id) = todo.pop() {
            if refs.contains(&id) {
                continue;
            }
            for cid in self.get(id).expect("is in store").ipld()?.references() {
                let id = self.lookup(&cid).ok_or_else(|| BlockNotFound(cid))?;
                todo.push(id);
            }
            refs.insert(id);
        }
        Ok(refs)
    }
}

#[derive(Default)]
struct BlockCache {
    next_atime: Atime,
    sorted: BTreeMap<Atime, Id>,
    atime: HashMap<Id, Atime>,
}

impl BlockCache {
    pub fn contains(&self, id: Id) -> bool {
        self.atime.contains_key(&id)
    }

    pub fn insert(&mut self, id: Id) {
        self.remove(id);
        let atime = self.next_atime;
        self.next_atime += 1;
        self.atime.insert(id, atime);
        self.sorted.insert(atime, id);
    }

    pub fn remove(&mut self, id: Id) -> Option<Atime> {
        if let Some(atime) = self.atime.remove(&id) {
            self.sorted.remove(&atime);
            Some(atime)
        } else {
            None
        }
    }

    pub fn hit(&mut self, id: Id) {
        if self.remove(id).is_some() {
            self.insert(id);
        }
    }

    pub fn evict(&mut self) -> Option<Id> {
        let id = self.sorted.iter().next().map(|(_, id)| *id);
        if let Some(id) = id {
            self.remove(id);
        }
        id
    }

    pub fn size(&self) -> usize {
        self.sorted.len()
    }
}

#[derive(Default)]
struct BlockAliases {
    aliases: HashMap<Vec<u8>, (Id, HashSet<Id>)>,
    refs: HashMap<Id, u64>,
}

impl BlockAliases {
    pub fn resolve(&self, alias: &[u8]) -> Option<Id> {
        self.aliases.get(alias).map(|(id, _)| *id)
    }

    pub fn alias(&mut self, alias: &[u8], id: Id, refs: HashSet<Id>) {
        for id in &refs {
            *self.refs.entry(*id).or_default() += 1;
        }
        self.aliases.insert(alias.to_vec(), (id, refs));
    }

    pub fn unalias(&mut self, alias: &[u8]) -> HashSet<Id> {
        let mut cache = HashSet::default();
        if let Some((_, refs)) = self.aliases.remove(alias) {
            for id in refs {
                let count = self.refs.get_mut(&id).expect("can't fail");
                *count -= 1;
                if *count < 1 {
                    cache.insert(id);
                }
            }
        }
        cache
    }
}

struct LocalStore<S: StoreParams> {
    cache_size: usize,
    blocks: BlockStore<S>,
    cache: BlockCache,
    aliases: BlockAliases,
}

impl<S: StoreParams> LocalStore<S> {
    pub fn new(cache_size: usize) -> Self {
        assert!(cache_size > 0);
        Self {
            cache_size,
            blocks: BlockStore::new(),
            cache: Default::default(),
            aliases: Default::default(),
        }
    }

    pub fn get(&mut self, cid: &Cid) -> Option<Block<S>> {
        if let Some(id) = self.blocks.lookup(cid) {
            self.cache.hit(id);
            self.blocks.get(id).cloned()
        } else {
            None
        }
    }

    pub fn insert(&mut self, block: Block<S>) {
        if let Some(id) = self.blocks.insert(block) {
            self.insert_cache(id);
        }
    }

    fn insert_cache(&mut self, id: Id) {
        self.cache.insert(id);
        if self.cache_size < self.cache.size() {
            if let Some(id) = self.cache.evict() {
                self.blocks.remove(id);
            }
        }
    }

    fn remove_cache(&mut self, id: Id) {
        self.cache.remove(id);
    }

    pub fn resolve<T: AsRef<[u8]>>(&self, alias: T) -> Option<Cid> {
        if let Some(id) = self.aliases.resolve(alias.as_ref()) {
            Some(*self.blocks.get(id).expect("can't fail").cid())
        } else {
            None
        }
    }

    pub fn alias<T: AsRef<[u8]>>(&mut self, alias: T, cid: Option<&Cid>) -> Result<()>
    where
        Ipld: Decode<S::Codecs>,
    {
        let alias = alias.as_ref();
        let ins = if let Some(cid) = cid {
            let id = self.blocks.lookup(cid).ok_or_else(|| BlockNotFound(*cid))?;
            let refs = self.blocks.references(id)?;
            Some((id, refs))
        } else {
            None
        };
        let mut cache = self.aliases.unalias(alias);
        if let Some((id, refs)) = ins {
            for id in &refs {
                self.remove_cache(*id);
                cache.remove(id);
            }
            self.aliases.alias(alias, id, refs);
        }
        for id in cache {
            self.insert_cache(id);
        }
        Ok(())
    }

    pub fn pinned(&self, cid: &Cid) -> Option<bool> {
        if let Some(id) = self.blocks.lookup(cid) {
            Some(!self.cache.contains(id))
        } else {
            None
        }
    }
}

/// Simulated network.
pub struct GlobalStore<S: StoreParams>(Arc<Mutex<HashSet<Block<S>>>>);

impl<S: StoreParams> Clone for GlobalStore<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: StoreParams> Default for GlobalStore<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S: StoreParams> GlobalStore<S> {
    /// Fetch a block from the network.
    pub fn get(&self, cid: &Cid) -> Option<Block<S>> {
        self.0.lock().unwrap().get(cid).cloned()
    }

    /// Insert a block in the network.
    pub fn insert(&self, block: Block<S>) {
        self.0.lock().unwrap().insert(block);
    }
}

struct SharedStore<S: StoreParams> {
    local: LocalStore<S>,
    network: GlobalStore<S>,
}

impl<S: StoreParams> SharedStore<S> {
    pub fn new(network: GlobalStore<S>, cache_size: usize) -> Self {
        Self {
            local: LocalStore::new(cache_size),
            network,
        }
    }

    pub fn get(&mut self, cid: &Cid) -> Option<Block<S>> {
        if let Some(block) = self.local.get(cid) {
            return Some(block);
        }
        if let Some(block) = self.network.get(cid) {
            self.local.insert(block.clone());
            return Some(block);
        }
        None
    }

    pub fn insert(&mut self, block: Block<S>) {
        self.local.insert(block.clone());
        self.network.insert(block);
    }

    pub fn resolve<T: AsRef<[u8]>>(&self, alias: T) -> Option<Cid> {
        self.local.resolve(alias)
    }

    pub fn alias<T: AsRef<[u8]>>(&mut self, alias: T, cid: Option<&Cid>) -> Result<()>
    where
        Ipld: Decode<S::Codecs>,
    {
        loop {
            if let Err(err) = self.local.alias(alias.as_ref(), cid) {
                if let Some(BlockNotFound(cid)) = err.downcast_ref::<BlockNotFound>() {
                    if self.get(cid).is_none() {
                        return Err(BlockNotFound(*cid).into());
                    }
                }
            } else {
                return Ok(());
            }
        }
    }

    pub fn pinned(&self, cid: &Cid) -> Option<bool> {
        self.local.pinned(cid)
    }
}

/// In memory reference store implementation. Is intended for testing.
pub struct MemStore<S: StoreParams>(Arc<Mutex<SharedStore<S>>>);

impl<S: StoreParams> Clone for MemStore<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: StoreParams> Default for MemStore<S> {
    fn default() -> Self {
        Self::new(Default::default(), 64)
    }
}

impl<S: StoreParams> MemStore<S> {
    /// Creates a new `MemStore`.
    pub fn new(network: GlobalStore<S>, cache_size: usize) -> Self {
        Self(Arc::new(Mutex::new(SharedStore::new(network, cache_size))))
    }

    /// Returns the status of a cid.
    ///
    /// not in store: None
    /// evictable: Some(false)
    /// not evictable: Some(true)
    pub fn pinned(&self, cid: &Cid) -> Option<bool> {
        self.0.lock().unwrap().pinned(cid)
    }
}

#[async_trait]
impl<S: StoreParams> Store for MemStore<S>
where
    Ipld: Decode<S::Codecs>,
{
    type Params = S;

    async fn get(&self, cid: &Cid) -> Result<Block<S>> {
        self.0
            .lock()
            .unwrap()
            .get(cid)
            .ok_or_else(|| BlockNotFound(*cid).into())
    }

    async fn insert(&self, block: &Block<S>) -> Result<()> {
        self.0.lock().unwrap().insert(block.clone());
        Ok(())
    }

    async fn alias<T: AsRef<[u8]> + Send + Sync>(&self, alias: T, cid: Option<&Cid>) -> Result<()> {
        self.0.lock().unwrap().alias(alias, cid)
    }

    async fn resolve<T: AsRef<[u8]> + Send + Sync>(&self, alias: T) -> Result<Option<Cid>> {
        Ok(self.0.lock().unwrap().resolve(alias))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor::DagCborCodec;
    use crate::multihash::SHA2_256;
    use crate::store::DefaultStoreParams;
    use crate::{alias, ipld};

    fn create_block(ipld: &Ipld) -> Block<DefaultStoreParams> {
        Block::encode(DagCborCodec, SHA2_256, ipld).unwrap()
    }

    macro_rules! assert_evicted {
        ($store:expr, $block:expr) => {
            assert_eq!($store.pinned($block.cid()), None);
        };
    }

    macro_rules! assert_pinned {
        ($store:expr, $block:expr) => {
            assert_eq!($store.pinned($block.cid()), Some(true));
        };
    }

    macro_rules! assert_unpinned {
        ($store:expr, $block:expr) => {
            assert_eq!($store.pinned($block.cid()), Some(false));
        };
    }

    #[test]
    fn test_store_evict() {
        let mut store = LocalStore::new(2);
        let blocks = [
            create_block(&ipld!(0)),
            create_block(&ipld!(1)),
            create_block(&ipld!(2)),
            create_block(&ipld!(3)),
        ];
        store.insert(blocks[0].clone());
        assert_unpinned!(&store, &blocks[0]);
        store.insert(blocks[1].clone());
        assert_unpinned!(&store, &blocks[1]);
        store.insert(blocks[2].clone());
        assert_evicted!(&store, &blocks[0]);
        assert_unpinned!(&store, &blocks[1]);
        assert_unpinned!(&store, &blocks[2]);
        store.get(&blocks[1]);
        store.insert(blocks[3].clone());
        assert_unpinned!(&store, &blocks[1]);
        assert_evicted!(&store, &blocks[2]);
        assert_unpinned!(&store, &blocks[3]);
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_store_unpin() {
        let mut store = LocalStore::new(3);
        let a = create_block(&ipld!({ "a": [] }));
        let b = create_block(&ipld!({ "b": [a.cid()] }));
        let c = create_block(&ipld!({ "c": [a.cid()] }));
        let x = alias!(x);
        let y = alias!(y);
        store.insert(a.clone());
        store.insert(b.clone());
        store.insert(c.clone());
        store.alias(x, Some(b.cid())).unwrap();
        store.alias(y, Some(c.cid())).unwrap();
        assert_pinned!(&store, &a);
        assert_pinned!(&store, &b);
        assert_pinned!(&store, &c);
        store.alias(x, None).unwrap();
        assert_pinned!(&store, &a);
        assert_unpinned!(&store, &b);
        assert_pinned!(&store, &c);
        store.alias(y, None).unwrap();
        assert_unpinned!(&store, &a);
        assert_unpinned!(&store, &b);
        assert_unpinned!(&store, &c);
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_store_unpin2() {
        let mut store = LocalStore::new(3);
        let a = create_block(&ipld!({ "a": [] }));
        let b = create_block(&ipld!({ "b": [a.cid()] }));
        let x = alias!(x);
        let y = alias!(y);
        store.insert(a.clone());
        store.insert(b.clone());
        store.alias(x, Some(b.cid())).unwrap();
        store.alias(y, Some(b.cid())).unwrap();
        assert_pinned!(&store, &a);
        assert_pinned!(&store, &b);
        store.alias(x, None).unwrap();
        assert_pinned!(&store, &a);
        assert_pinned!(&store, &b);
        store.alias(y, None).unwrap();
        assert_unpinned!(&store, &a);
        assert_unpinned!(&store, &b);
    }

    #[test]
    fn test_sync() {
        let network = GlobalStore::default();
        let mut local1 = SharedStore::new(network.clone(), 5);
        let mut local2 = SharedStore::new(network, 5);
        let a1 = create_block(&ipld!({ "a": 0 }));
        let b1 = create_block(&ipld!({ "b": 0 }));
        let c1 = create_block(&ipld!({ "c": [a1.cid(), b1.cid()] }));
        let b2 = create_block(&ipld!({ "b": 1 }));
        let c2 = create_block(&ipld!({ "c": [a1.cid(), b2.cid()] }));
        let x = alias!(x);

        local1.insert(a1.clone());
        local1.insert(b1.clone());
        local1.insert(c1.clone());
        local1.alias(x, Some(c1.cid())).unwrap();
        assert_pinned!(&local1, &a1);
        assert_pinned!(&local1, &b1);
        assert_pinned!(&local1, &c1);

        local2.alias(x, Some(c1.cid())).unwrap();
        assert_pinned!(&local2, &a1);
        assert_pinned!(&local2, &b1);
        assert_pinned!(&local2, &c1);

        local2.insert(b2.clone());
        local2.insert(c2.clone());
        local2.alias(x, Some(c2.cid())).unwrap();
        assert_pinned!(&local2, &a1);
        assert_unpinned!(&local2, &b1);
        assert_unpinned!(&local2, &c1);
        assert_pinned!(&local2, &b2);
        assert_pinned!(&local2, &c2);

        local1.alias(x, Some(c2.cid())).unwrap();
        assert_pinned!(&local1, &a1);
        assert_unpinned!(&local1, &b1);
        assert_unpinned!(&local1, &c1);
        assert_pinned!(&local1, &b2);
        assert_pinned!(&local1, &c2);

        local2.alias(x, None).unwrap();
        assert_unpinned!(&local2, &a1);
        assert_unpinned!(&local2, &b1);
        assert_unpinned!(&local2, &c1);
        assert_unpinned!(&local2, &b2);
        assert_unpinned!(&local2, &c2);

        local1.alias(x, None).unwrap();
        assert_unpinned!(&local1, &a1);
        assert_unpinned!(&local1, &b1);
        assert_unpinned!(&local1, &c1);
        assert_unpinned!(&local1, &b2);
        assert_unpinned!(&local1, &c2);
    }
}
