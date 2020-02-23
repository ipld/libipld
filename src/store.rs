//! Traits for implementing a block store.
use crate::block::{create_cbor_block, decode_cbor, decode_ipld, validate};
use crate::cid::Cid;
use crate::error::Result;
use crate::gc::closure;
use crate::hash::{CidHashMap, CidHashSet, Hash};
use crate::ipld::Ipld;
use core::ops::Deref;
use dag_cbor::{ReadCbor, WriteCbor};
use std::collections::HashMap;
use std::mem;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

/// Implementable by ipld storage backends.

pub trait Store: Send + Sync {
    /// Returns the block with cid.
    fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>>;
    /// Writes the block with cid.
    fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()>;
    /// Flushes the write buffer.
    fn flush(&self) -> Result<()>;
    /// Gc's unused blocks.
    fn gc(&self) -> Result<()>;

    /// Pin a block.
    fn pin(&self, cid: &Cid) -> Result<()>;
    /// Unpin a block.
    fn unpin(&self, cid: &Cid) -> Result<()>;
    /// Create an indirect user managed pin.
    fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()>;

    /// Write a link to a block.
    fn write_link(&self, label: &str, cid: &Cid) -> Result<()>;
    /// Read a link to a block.
    fn read_link(&self, label: &str) -> Result<Option<Cid>>;
    /// Remove link to a block.
    fn remove_link(&self, label: &str) -> Result<()>;
}

impl<TStore: Store> Store for Arc<TStore> {
    fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        self.deref().read(cid)
    }

    fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        self.deref().write(cid, data)
    }

    fn flush(&self) -> Result<()> {
        self.deref().flush()
    }

    fn gc(&self) -> Result<()> {
        self.deref().gc()
    }

    fn pin(&self, cid: &Cid) -> Result<()> {
        self.deref().pin(cid)
    }

    fn unpin(&self, cid: &Cid) -> Result<()> {
        self.deref().unpin(cid)
    }

    fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()> {
        self.deref().autopin(cid, auto_path)
    }

    fn write_link(&self, label: &str, cid: &Cid) -> Result<()> {
        self.deref().write_link(label, cid)
    }

    fn read_link(&self, label: &str) -> Result<Option<Cid>> {
        self.deref().read_link(label)
    }

    fn remove_link(&self, label: &str) -> Result<()> {
        self.deref().remove_link(label)
    }
}

/// A store wrapper for debugging.
pub struct DebugStore<TStore: Store> {
    prefix: &'static str,
    store: TStore,
}

fn print_cid(cid: &Cid) -> String {
    (&cid.to_string()[..30]).to_string()
}

impl<TStore: Store> DebugStore<TStore> {
    /// Creates a new debug store.
    pub fn new(store: TStore) -> Self {
        Self::new_with_prefix(store, "")
    }

    /// Creates a new debug store.
    pub fn new_with_prefix(store: TStore, prefix: &'static str) -> Self {
        Self { store, prefix }
    }
}

impl<TStore: Store> Store for DebugStore<TStore> {
    fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        let res = self.store.read(cid)?;
        println!(
            "{}read {} {:?}",
            self.prefix,
            print_cid(cid),
            res.as_ref().map(|d| d.len())
        );
        Ok(res)
    }

    fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        println!("{}write {} {}", self.prefix, print_cid(cid), data.len());
        self.store.write(cid, data)
    }

    fn flush(&self) -> Result<()> {
        println!("{}flush", self.prefix);
        self.store.flush()
    }

    fn gc(&self) -> Result<()> {
        println!("{}gc", self.prefix);
        self.store.gc()
    }

    fn pin(&self, cid: &Cid) -> Result<()> {
        println!("{}pin {}", self.prefix, print_cid(cid));
        self.store.pin(cid)
    }

    fn unpin(&self, cid: &Cid) -> Result<()> {
        println!("{}unpin {}", self.prefix, print_cid(cid));
        self.store.unpin(cid)
    }

    fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()> {
        println!("{}autopin {}", self.prefix, print_cid(cid));
        self.store.autopin(cid, auto_path)
    }

    fn write_link(&self, label: &str, cid: &Cid) -> Result<()> {
        println!("{}write_link {} {}", self.prefix, label, print_cid(cid));
        self.store.write_link(label, cid)
    }

    fn read_link(&self, label: &str) -> Result<Option<Cid>> {
        let res = self.store.read_link(label)?;
        println!(
            "{}read_link {} {:?}",
            self.prefix,
            label,
            res.as_ref().map(print_cid)
        );
        Ok(res)
    }

    fn remove_link(&self, label: &str) -> Result<()> {
        println!("{}remove_link {}", self.prefix, label);
        self.store.remove_link(label)
    }
}

/// Ipld extension trait.

pub trait StoreIpldExt {
    /// Reads the block with cid and decodes it to ipld.
    fn read_ipld(&self, cid: &Cid) -> Result<Option<Ipld>>;
}

impl<T: Store> StoreIpldExt for T {
    fn read_ipld(&self, cid: &Cid) -> Result<Option<Ipld>> {
        if let Some(data) = self.read(cid)? {
            let ipld = decode_ipld(cid, &data)?;
            return Ok(Some(ipld));
        }
        Ok(None)
    }
}

/// Cbor extension trait.
pub trait StoreCborExt {
    /// Reads the block with cid and decodes it to cbor.
    fn read_cbor<C: ReadCbor + Send>(&self, cid: &Cid) -> Result<Option<C>>;

    /// Writes a block using the cbor codec.
    fn write_cbor<H: Hash, C: WriteCbor + Send + Sync>(&self, c: &C) -> Result<Cid>;
}

impl<T: Store> StoreCborExt for T {
    fn read_cbor<C: ReadCbor + Send>(&self, cid: &Cid) -> Result<Option<C>> {
        if let Some(data) = self.read(cid)? {
            let cbor = decode_cbor::<C>(cid, &data)?;
            return Ok(Some(cbor));
        }
        Ok(None)
    }

    fn write_cbor<H: Hash, C: WriteCbor + Send + Sync>(&self, c: &C) -> Result<Cid> {
        let (cid, data) = create_cbor_block::<H, C>(c)?;
        self.write(&cid, data)?;
        Ok(cid)
    }
}

/// A memory backed store
#[derive(Default)]
pub struct MemStore {
    blocks: RwLock<CidHashMap<Box<[u8]>>>,
    pins: RwLock<CidHashSet>,
    links: RwLock<HashMap<String, Cid>>,
}

impl Store for MemStore {
    fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        Ok(self.blocks.read()?.get(cid).cloned())
    }

    fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        self.blocks.write()?.insert(cid.clone(), data);
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }

    fn gc(&self) -> Result<()> {
        let pins = self.pins.read()?;
        let roots = pins.iter().map(Clone::clone).collect();
        let blocks = self
            .blocks
            .read()?
            .iter()
            .map(|(cid, _)| cid.clone())
            .collect();
        let dead = crate::gc::dead_paths(self, blocks, roots)?;
        for cid in dead {
            self.blocks.write()?.remove(&cid);
        }
        Ok(())
    }

    fn pin(&self, cid: &Cid) -> Result<()> {
        self.pins.write()?.insert(cid.clone());
        Ok(())
    }

    fn unpin(&self, cid: &Cid) -> Result<()> {
        self.pins.write()?.remove(&cid);
        Ok(())
    }

    fn autopin(&self, cid: &Cid, _: &Path) -> Result<()> {
        self.pin(cid)
    }

    fn write_link(&self, link: &str, cid: &Cid) -> Result<()> {
        self.links.write()?.insert(link.to_string(), cid.clone());
        Ok(())
    }

    fn read_link(&self, link: &str) -> Result<Option<Cid>> {
        Ok(self.links.read()?.get(link).cloned())
    }

    fn remove_link(&self, link: &str) -> Result<()> {
        self.links.write()?.remove(link);
        Ok(())
    }
}

/// A buffered store.
pub struct BufStore<TStore: Store = MemStore> {
    store: TStore,
    cache: RwLock<CidHashMap<Box<[u8]>>>,
    buffer: RwLock<CidHashMap<Box<[u8]>>>,
    pins: RwLock<CidHashMap<PinOp>>,
}

#[derive(Debug)]
enum PinOp {
    Pin,
    Unpin,
}

impl<TStore: Store> BufStore<TStore> {
    /// Creates a new block store.
    pub fn new(store: TStore, _cache_cap: usize, _buffer_cap: usize) -> Self {
        Self {
            store,
            cache: Default::default(),
            buffer: Default::default(),
            pins: Default::default(),
        }
    }
}

impl<TStore: Store> Store for BufStore<TStore> {
    fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        let cached = self.cache.read()?.get(cid).cloned();
        if let Some(data) = cached {
            return Ok(Some(data));
        }
        let fresh = self.store.read(cid)?;
        if let Some(ref data) = fresh {
            validate(cid, &data)?;
            self.cache.write()?.insert(cid.clone(), data.clone());
        }
        Ok(fresh)
    }

    fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        self.cache.write()?.insert(cid.clone(), data.clone());
        self.buffer.write()?.insert(cid.clone(), data);
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        let (pins, buffer) = {
            let (mut pins, mut buffer) = (self.pins.write()?, self.buffer.write()?);
            let pins = mem::replace(&mut *pins, Default::default());
            let buffer = mem::replace(&mut *buffer, Default::default());
            (pins, buffer)
        };

        let mut buffer_pins: CidHashSet = Default::default();
        for (cid, op) in pins.into_iter() {
            if buffer.contains_key(&cid) {
                if let PinOp::Pin = op {
                    buffer_pins.insert(cid);
                }
            } else {
                match op {
                    PinOp::Pin => self.store.pin(&cid)?,
                    PinOp::Unpin => self.store.unpin(&cid)?,
                }
            }
        }

        let live = closure(self, buffer_pins.clone())?;
        for (cid, data) in buffer {
            if live.contains(&cid) {
                self.store.write(&cid, data)?;
            }
            if buffer_pins.contains(&cid) {
                self.store.pin(&cid)?;
            }
        }
        self.store.flush()?;

        Ok(())
    }

    fn gc(&self) -> Result<()> {
        self.store.gc()
    }

    fn pin(&self, cid: &Cid) -> Result<()> {
        self.pins.write()?.insert(cid.clone(), PinOp::Pin);
        Ok(())
    }

    fn unpin(&self, cid: &Cid) -> Result<()> {
        self.pins.write()?.insert(cid.clone(), PinOp::Unpin);
        Ok(())
    }

    fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()> {
        self.store.autopin(cid, auto_path)
    }

    fn write_link(&self, label: &str, cid: &Cid) -> Result<()> {
        self.store.write_link(label, cid)
    }

    fn read_link(&self, label: &str) -> Result<Option<Cid>> {
        self.store.read_link(label)
    }

    fn remove_link(&self, label: &str) -> Result<()> {
        self.store.remove_link(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{create_cbor_block, create_raw_block};
    use crate::DefaultHash as H;
    use model::*;

    fn create_block_raw(n: usize) -> (Cid, Box<[u8]>) {
        let data = n.to_ne_bytes().to_vec().into_boxed_slice();
        create_raw_block::<H>(data).unwrap()
    }

    fn create_block_ipld(ipld: &Ipld) -> (Cid, Box<[u8]>) {
        create_cbor_block::<H, _>(ipld).unwrap()
    }

    #[test]
    fn test_obj() {
        let store = MemStore::default();
        let _ = &store as &dyn Store;
        let store = Arc::new(store);
        let _ = &store as &dyn Store;

        let store = MemStore::default();
        let store = BufStore::new(store, 16, 16);
        let _ = &store as &dyn Store;
        let store = Arc::new(store);
        let _ = &store as &dyn Store;
    }

    fn run_gc_no_pin(store: &dyn Store) {
        let (cid_0, data_0) = create_block_raw(0);
        store.write(&cid_0, data_0).unwrap();
        store.flush().unwrap();
        store.gc().unwrap();
        let data_0_2 = store.read(&cid_0).unwrap();
        assert!(data_0_2.is_none());
    }

    #[test]
    fn test_gc_no_pin() {
        let store = MemStore::default();
        run_gc_no_pin(&store);
    }

    fn run_gc_pin(store: &dyn Store) {
        let (cid_0, data_0) = create_block_raw(0);
        store.write(&cid_0, data_0.clone()).unwrap();
        store.pin(&cid_0).unwrap();
        store.flush().unwrap();
        store.gc().unwrap();
        let data_0_2 = store.read(&cid_0).unwrap();
        assert_eq!(data_0_2, Some(data_0));
    }

    #[test]
    fn test_gc_pin() {
        let store = MemStore::default();
        run_gc_pin(&store);
    }

    fn run_gc_pin_leaf(store: &dyn Store) {
        let (cid_0, data_0) = create_block_raw(0);
        let ipld = Ipld::Link(cid_0.clone());
        let (cid_1, data_1) = create_block_ipld(&ipld);
        store.write(&cid_0, data_0.clone()).unwrap();
        store.write(&cid_1, data_1.clone()).unwrap();
        store.pin(&cid_1).unwrap();
        store.flush().unwrap();
        store.gc().unwrap();
        let data_0_2 = store.read(&cid_0).unwrap();
        assert_eq!(data_0_2, Some(data_0));
    }

    #[test]
    fn test_gc_pin_leaf() {
        let store = MemStore::default();
        run_gc_pin_leaf(&store);
    }

    #[test]
    fn mem_buf_store_eqv() {
        const LEN: usize = 4;
        let blocks: Vec<_> = (0..LEN).into_iter().map(create_block_raw).collect();
        model! {
            Model => let mem_store = MemStore::default(),
            Implementation => let buf_store = BufStore::new(MemStore::default(), 16, 16),
            Read(usize)(i in 0..LEN) => {
                let (cid, _) = &blocks[i];
                let mem = mem_store.read(cid).unwrap();
                let buf = buf_store.read(cid).unwrap();
                // Element can be in cache after gc.
                if !(mem.is_none() && buf.is_some()) {
                    assert_eq!(mem, buf);
                }
            },
            Write(usize)(i in 0..LEN) => {
                let (cid, data) = &blocks[i];
                let mem = mem_store.write(cid, data.clone()).unwrap();
                let buf = buf_store.write(cid, data.clone()).unwrap();
                (mem, buf);
            },
            Flush(usize)(_ in 0..LEN) => {
                let mem = mem_store.flush().unwrap();
                let buf = buf_store.flush().unwrap();
                (mem, buf);
            },
            Gc(usize)(_ in 0..LEN) => {
                let mem = mem_store.gc().unwrap();
                let buf = buf_store.gc().unwrap();
                (mem, buf);
            },
            Pin(usize)(i in 0..LEN) => {
                let (cid, _) = &blocks[i];
                let mem = mem_store.pin(&cid).unwrap();
                let buf = buf_store.pin(&cid).unwrap();
                (mem, buf);
            },
            Unpin(usize)(i in 0..LEN) => {
                let (cid, _) = &blocks[i];
                let mem = mem_store.unpin(&cid).unwrap();
                let buf = buf_store.unpin(&cid).unwrap();
                (mem, buf);
            }
        }
    }

    macro_rules! linearizable_store {
        ($store:expr) => {
            const LEN: usize = 4;
            let blocks: Vec<_> = (0..LEN).into_iter().map(create_block_raw).collect();
            let blocks = Shared::new(blocks);
            const LLEN: usize = 3;
            let links = Shared::new(["a", "b", "c"]);
            linearizable! {
                Implementation => let store = model::Shared::new($store),
                Read(usize)(i in 0..LEN) -> Option<Box<[u8]>> {
                    let (cid, _) = &blocks[i];
                    store.read(cid).unwrap()
                },
                Write(usize)(i in 0..LEN) -> () {
                    let (cid, data) = &blocks[i];
                    store.write(cid, data.clone()).unwrap()
                },
                Flush(usize)(_ in 0..LEN) -> () {
                    store.flush().unwrap()
                },
                Gc(usize)(_ in 0..LEN) -> () {
                    store.gc().unwrap()
                },
                Pin(usize)(i in 0..LEN) -> () {
                    let (cid, _) = &blocks[i];
                    store.pin(cid).unwrap()
                },
                Unpin(usize)(i in 0..LEN) -> () {
                    let (cid, _) = &blocks[i];
                    store.unpin(cid).unwrap()
                },
                WriteLink((usize, usize))((i1, i2) in (0..LLEN, 0..LEN)) -> () {
                    let link = &links[i1];
                    let (cid, _) = &blocks[i2];
                    store.write_link(link, cid).unwrap()
                },
                ReadLink(usize)(i in 0..LLEN) -> Option<Cid> {
                    let link = &links[i];
                    store.read_link(link).unwrap()
                },
                RemoveLink(usize)(i in 0..LLEN) -> () {
                    let link = &links[i];
                    store.remove_link(link).unwrap()
                }
            }
        };
    }

    #[test]
    fn mem_store_lin() {
        linearizable_store!(MemStore::default());
    }

    #[test]
    fn buf_store_lin() {
        linearizable_store!(BufStore::new(MemStore::default(), 16, 16));
    }
}
