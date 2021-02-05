//! Reference store implementation.
use crate::block::Block;
use crate::cid::Cid;
use crate::codec::References;
use crate::error::{BlockNotFound, Result};
use crate::ipld::Ipld;
use crate::store::{Store, StoreParams};
use async_trait::async_trait;
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// Temp pin.
#[derive(Clone)]
pub struct TempPin(Arc<InnerTempPin>);

struct InnerTempPin {
    id: u64,
    temp_pins: Arc<Mutex<FnvHashMap<u64, Vec<Id>>>>,
}

impl std::fmt::Debug for TempPin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TempPin({})", self.0.id)
    }
}

impl Drop for InnerTempPin {
    fn drop(&mut self) {
        self.temp_pins.lock().unwrap().remove(&self.id);
    }
}

type Id = u64;
type Atime = u64;

struct LocalStore<S: StoreParams> {
    _marker: PhantomData<S>,
    next_id: Id,
    cid: FnvHashMap<Id, Cid>,
    data: FnvHashMap<Id, Vec<u8>>,
    lookup: FnvHashMap<Cid, Id>,
    refs: FnvHashMap<Id, Vec<Id>>,

    cache_size: usize,
    next_atime: Atime,
    sorted: BTreeMap<Atime, Id>,
    atime: FnvHashMap<Id, Atime>,

    next_temp_pin: u64,
    temp_pins: Arc<Mutex<FnvHashMap<u64, Vec<Id>>>>,
    aliases: FnvHashMap<Vec<u8>, Id>,
}

impl<S: StoreParams> LocalStore<S> {
    pub fn new(cache_size: usize) -> Self {
        Self {
            _marker: Default::default(),
            next_id: 0,
            cid: Default::default(),
            data: Default::default(),
            lookup: Default::default(),
            refs: Default::default(),

            cache_size,
            next_atime: 0,
            sorted: Default::default(),
            atime: Default::default(),

            next_temp_pin: 0,
            temp_pins: Default::default(),
            aliases: Default::default(),
        }
    }

    fn lookup(&mut self, cid: &Cid) -> Id {
        if let Some(id) = self.lookup.get(cid) {
            *id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.lookup.insert(*cid, id);
            self.cid.insert(id, *cid);
            id
        }
    }

    fn contains(&mut self, cid: &Cid) -> bool {
        let id = self.lookup(cid);
        self.data.contains_key(&id)
    }

    fn create_temp_pin(&mut self) -> TempPin {
        let id = self.next_temp_pin;
        self.next_temp_pin += 1;
        TempPin(Arc::new(InnerTempPin {
            id,
            temp_pins: self.temp_pins.clone(),
        }))
    }

    fn temp_pin(&mut self, tmp: &TempPin, cid: &Cid) {
        let id = self.lookup(cid);
        self.temp_pins
            .lock()
            .unwrap()
            .entry(tmp.0.id)
            .or_default()
            .push(id);
    }

    fn hit(&mut self, id: Id) {
        if let Some(atime) = self.atime.remove(&id) {
            self.sorted.remove(&atime);
        }
        let atime = self.next_atime;
        self.next_atime += 1;
        self.sorted.insert(atime, id);
        self.atime.insert(id, atime);
    }

    fn get(&mut self, cid: &Cid) -> Option<Block<S>> {
        let id = self.lookup(cid);
        self.hit(id);
        let cid = *self.cid.get(&id)?;
        let data = self.data.get(&id)?.clone();
        Some(Block::new_unchecked(cid, data))
    }

    fn insert(&mut self, block: Block<S>) -> Result<()>
    where
        Ipld: References<S::Codecs>,
    {
        let id = self.lookup(block.cid());
        let mut refs = FnvHashSet::default();
        block.references(&mut refs)?;
        let (_cid, data) = block.into_inner();
        let ids = refs.iter().map(|id| self.lookup(id)).collect();
        self.refs.insert(id, ids);
        self.data.insert(id, data);
        self.hit(id);
        Ok(())
    }

    pub fn resolve<T: AsRef<[u8]>>(&mut self, alias: T) -> Option<Cid> {
        if let Some(id) = self.aliases.get(alias.as_ref()) {
            Some(*self.cid.get(id).unwrap())
        } else {
            None
        }
    }

    pub fn alias<T: AsRef<[u8]>>(&mut self, alias: T, cid: Option<&Cid>) {
        if let Some(cid) = cid {
            let id = self.lookup(cid);
            self.aliases.insert(alias.as_ref().to_vec(), id);
        } else {
            self.aliases.remove(alias.as_ref());
        }
    }

    pub fn reverse_alias(&mut self, cid: &Cid) -> Option<Vec<Vec<u8>>> {
        let id = self.lookup(cid);
        if self.data.contains_key(&id) {
            let mut aliases = vec![];
            for (alias, root) in &self.aliases {
                let closure = self.closure(vec![*root]);
                if closure.contains(&id) {
                    aliases.push(alias.clone());
                }
            }
            Some(aliases)
        } else {
            None
        }
    }

    pub fn evict(&mut self) {
        let mut n = self.data.len() as i64 - self.cache_size as i64;
        if n <= 0 {
            return;
        }
        let roots = self.roots();
        let pinned = self.closure(roots);
        let mut remove = Vec::with_capacity(n as usize);
        for (atime, id) in self.sorted.iter() {
            if n <= 0 {
                break;
            }

            if pinned.contains(id) {
                continue;
            }
            n -= 1;
            remove.push(*atime);
            self.cid.remove(&id);
            self.data.remove(&id);
            self.refs.remove(&id);
            self.atime.remove(&id);
        }
        for atime in remove {
            self.sorted.remove(&atime);
        }
    }

    pub fn roots(&self) -> Vec<Id> {
        let mut roots = vec![];
        for id in self.aliases.values() {
            roots.push(*id);
        }
        for ids in self.temp_pins.lock().unwrap().values() {
            for id in ids {
                roots.push(*id);
            }
        }
        roots
    }

    pub fn closure(&self, mut roots: Vec<Id>) -> FnvHashSet<Id> {
        let mut pinned = FnvHashSet::default();
        while let Some(id) = roots.pop() {
            if pinned.contains(&id) {
                continue;
            }
            if let Some(refs) = self.refs.get(&id) {
                for id in refs {
                    roots.push(*id);
                }
            }
            pinned.insert(id);
        }
        pinned
    }

    pub fn pinned(&mut self, cid: &Cid) -> Option<bool> {
        let aliases = self.reverse_alias(cid)?;
        Some(!aliases.is_empty())
    }
}

/// Simulated network.
pub struct GlobalStore<S: StoreParams>(Arc<Mutex<FnvHashSet<Block<S>>>>);

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

    pub fn create_temp_pin(&mut self) -> TempPin {
        self.local.create_temp_pin()
    }

    pub fn temp_pin(&mut self, tmp: &TempPin, cid: &Cid) {
        self.local.temp_pin(tmp, cid)
    }

    pub fn contains(&mut self, cid: &Cid) -> bool {
        self.local.contains(cid)
    }

    pub fn get(&mut self, cid: &Cid) -> Result<Block<S>>
    where
        Ipld: References<S::Codecs>,
    {
        if let Some(block) = self.local.get(cid) {
            Ok(block)
        } else {
            Err(BlockNotFound(*cid).into())
        }
    }

    pub fn fetch(&mut self, cid: &Cid) -> Result<Block<S>>
    where
        Ipld: References<S::Codecs>,
    {
        if let Some(block) = self.local.get(cid) {
            return Ok(block);
        }
        if let Some(block) = self.network.get(cid) {
            self.local.insert(block.clone())?;
            Ok(block)
        } else {
            Err(BlockNotFound(*cid).into())
        }
    }

    pub fn sync(&mut self, cid: &Cid) -> Result<()>
    where
        Ipld: References<S::Codecs>,
    {
        let id = self.local.lookup(cid);
        let mut missing = vec![id];
        while let Some(id) = missing.pop() {
            if !self.local.data.contains_key(&id) {
                let cid = *self.local.cid.get(&id).unwrap();
                self.fetch(&cid)?;
            }
            for id in self.local.refs.get(&id).unwrap() {
                missing.push(*id);
            }
        }
        Ok(())
    }

    pub fn insert(&mut self, block: Block<S>) -> Result<()>
    where
        Ipld: References<S::Codecs>,
    {
        self.local.insert(block.clone())?;
        self.network.insert(block);
        Ok(())
    }

    pub fn resolve<T: AsRef<[u8]>>(&mut self, alias: T) -> Option<Cid> {
        self.local.resolve(alias)
    }

    pub fn alias<T: AsRef<[u8]>>(&mut self, alias: T, cid: Option<&Cid>) {
        self.local.alias(alias.as_ref(), cid)
    }

    pub fn reverse_alias(&mut self, cid: &Cid) -> Option<Vec<Vec<u8>>> {
        self.local.reverse_alias(cid)
    }

    pub fn pinned(&mut self, cid: &Cid) -> Option<bool> {
        self.local.pinned(cid)
    }

    pub fn evict(&mut self) {
        self.local.evict()
    }
}

/// In memory reference store implementation. Is intended for testing.
#[derive(Clone)]
pub struct MemStore<S: StoreParams>(Arc<Mutex<SharedStore<S>>>);

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

    /// Evicts blocks from the memstore.
    pub fn evict(&self) {
        self.0.lock().unwrap().evict();
    }

    /// Checks if a block is pinned.
    pub fn pinned(&self, cid: &Cid) -> Option<bool> {
        self.0.lock().unwrap().pinned(cid)
    }
}

#[async_trait]
impl<S: StoreParams> Store for MemStore<S>
where
    Ipld: References<S::Codecs>,
{
    type Params = S;
    type TempPin = TempPin;

    fn create_temp_pin(&self) -> Result<Self::TempPin> {
        Ok(self.0.lock().unwrap().create_temp_pin())
    }

    fn temp_pin(&self, tmp: &Self::TempPin, cid: &Cid) -> Result<()> {
        self.0.lock().unwrap().temp_pin(tmp, cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool> {
        Ok(self.0.lock().unwrap().contains(cid))
    }

    fn get(&self, cid: &Cid) -> Result<Block<S>> {
        self.0.lock().unwrap().get(cid)
    }

    fn insert(&self, block: &Block<S>) -> Result<()> {
        self.0.lock().unwrap().insert(block.clone())
    }

    fn alias<T: AsRef<[u8]> + Send + Sync>(&self, alias: T, cid: Option<&Cid>) -> Result<()> {
        self.0.lock().unwrap().alias(alias, cid);
        Ok(())
    }

    fn resolve<T: AsRef<[u8]> + Send + Sync>(&self, alias: T) -> Result<Option<Cid>> {
        Ok(self.0.lock().unwrap().resolve(alias))
    }

    fn reverse_alias(&self, cid: &Cid) -> Result<Option<Vec<Vec<u8>>>> {
        Ok(self.0.lock().unwrap().reverse_alias(cid))
    }

    async fn fetch(&self, cid: &Cid) -> Result<Block<S>> {
        self.0.lock().unwrap().fetch(cid)
    }

    async fn sync(&self, cid: &Cid) -> Result<()> {
        self.0.lock().unwrap().sync(cid)
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor::DagCborCodec;
    use crate::multihash::Code;
    use crate::store::DefaultParams;
    use crate::{alias, ipld};

    fn create_block(ipld: &Ipld) -> Block<DefaultParams> {
        Block::encode(DagCborCodec, Code::Blake3_256, ipld).unwrap()
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
    fn test_store_evict() -> Result<()> {
        let mut store = LocalStore::new(2);
        let blocks = [
            create_block(&ipld!(0)),
            create_block(&ipld!(1)),
            create_block(&ipld!(2)),
            create_block(&ipld!(3)),
        ];
        store.insert(blocks[0].clone())?;
        assert_unpinned!(store, &blocks[0]);
        store.insert(blocks[1].clone())?;
        assert_unpinned!(store, &blocks[1]);
        store.insert(blocks[2].clone())?;
        store.evict();
        assert_evicted!(store, &blocks[0]);
        assert_unpinned!(store, &blocks[1]);
        assert_unpinned!(store, &blocks[2]);
        store.get(&blocks[1]);
        store.insert(blocks[3].clone())?;
        store.evict();
        assert_unpinned!(store, &blocks[1]);
        assert_evicted!(store, &blocks[2]);
        assert_unpinned!(store, &blocks[3]);
        Ok(())
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_store_unpin() -> Result<()> {
        let mut store = LocalStore::new(3);
        let a = create_block(&ipld!({ "a": [] }));
        let b = create_block(&ipld!({ "b": [a.cid()] }));
        let c = create_block(&ipld!({ "c": [a.cid()] }));
        let x = alias!(x);
        let y = alias!(y);
        store.insert(a.clone())?;
        store.insert(b.clone())?;
        store.insert(c.clone())?;
        store.alias(x, Some(b.cid()));
        store.alias(y, Some(c.cid()));
        assert_pinned!(store, &a);
        assert_pinned!(store, &b);
        assert_pinned!(store, &c);
        store.alias(x, None);
        assert_pinned!(store, &a);
        assert_unpinned!(store, &b);
        assert_pinned!(store, &c);
        store.alias(y, None);
        assert_unpinned!(store, &a);
        assert_unpinned!(store, &b);
        assert_unpinned!(store, &c);
        Ok(())
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_store_unpin2() -> Result<()> {
        let mut store = LocalStore::new(3);
        let a = create_block(&ipld!({ "a": [] }));
        let b = create_block(&ipld!({ "b": [a.cid()] }));
        let x = alias!(x);
        let y = alias!(y);
        store.insert(a.clone())?;
        store.insert(b.clone())?;
        store.alias(x, Some(b.cid()));
        store.alias(y, Some(b.cid()));
        assert_pinned!(store, &a);
        assert_pinned!(store, &b);
        store.alias(x, None);
        assert_pinned!(store, &a);
        assert_pinned!(store, &b);
        store.alias(y, None);
        assert_unpinned!(store, &a);
        assert_unpinned!(store, &b);
        Ok(())
    }

    #[test]
    fn test_sync() -> Result<()> {
        let network = GlobalStore::default();
        let mut local1 = SharedStore::new(network.clone(), 5);
        let mut local2 = SharedStore::new(network, 5);
        let a1 = create_block(&ipld!({ "a": 0 }));
        let b1 = create_block(&ipld!({ "b": 0 }));
        let c1 = create_block(&ipld!({ "c": [a1.cid(), b1.cid()] }));
        let b2 = create_block(&ipld!({ "b": 1 }));
        let c2 = create_block(&ipld!({ "c": [a1.cid(), b2.cid()] }));
        let x = alias!(x);

        local1.insert(a1.clone())?;
        local1.insert(b1.clone())?;
        local1.insert(c1.clone())?;
        local1.alias(x, Some(c1.cid()));
        assert_pinned!(local1, &a1);
        assert_pinned!(local1, &b1);
        assert_pinned!(local1, &c1);

        local2.alias(x, Some(c1.cid()));
        local2.sync(c1.cid()).unwrap();
        assert_pinned!(local2, &a1);
        assert_pinned!(local2, &b1);
        assert_pinned!(local2, &c1);

        local2.insert(b2.clone())?;
        local2.insert(c2.clone())?;
        local2.alias(x, Some(c2.cid()));
        assert_pinned!(local2, &a1);
        assert_unpinned!(local2, &b1);
        assert_unpinned!(local2, &c1);
        assert_pinned!(local2, &b2);
        assert_pinned!(local2, &c2);

        local1.alias(x, Some(c2.cid()));
        local1.sync(c2.cid()).unwrap();
        assert_pinned!(local1, &a1);
        assert_unpinned!(local1, &b1);
        assert_unpinned!(local1, &c1);
        assert_pinned!(local1, &b2);
        assert_pinned!(local1, &c2);

        local2.alias(x, None);
        assert_unpinned!(local2, &a1);
        assert_unpinned!(local2, &b1);
        assert_unpinned!(local2, &c1);
        assert_unpinned!(local2, &b2);
        assert_unpinned!(local2, &c2);

        local1.alias(x, None);
        assert_unpinned!(local1, &a1);
        assert_unpinned!(local1, &b1);
        assert_unpinned!(local1, &c1);
        assert_unpinned!(local1, &b2);
        assert_unpinned!(local1, &c2);
        Ok(())
    }
}
