use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libipld::block::create_raw_block;
use libipld::error::Result;
use libipld::store::{BufStore, MemStore, Store};
use libipld::{Cid, DefaultHash as H};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

fn gen_block(n: usize) -> (Cid, Box<[u8]>) {
    let data = n.to_ne_bytes().to_vec().into_boxed_slice();
    create_raw_block::<H>(data).unwrap()
}

struct DelayStore<TStore: Store>(TStore);

impl<TStore: Store> DelayStore<TStore> {
    pub fn new(store: TStore) -> Self {
        Self(store)
    }
}

impl<TStore: Store> Store for DelayStore<TStore> {
    fn read(&self, cid: &Cid) -> Result<Option<Box<[u8]>>> {
        sleep(Duration::from_millis(1));
        self.0.read(cid)
    }

    fn write(&self, cid: &Cid, data: Box<[u8]>) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.write(cid, data)
    }

    fn flush(&self) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.flush()
    }

    fn gc(&self) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.gc()
    }

    fn pin(&self, cid: &Cid) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.pin(cid)
    }

    fn unpin(&self, cid: &Cid) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.unpin(cid)
    }

    fn autopin(&self, cid: &Cid, auto_path: &Path) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.autopin(cid, auto_path)
    }

    fn write_link(&self, label: &str, cid: &Cid) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.write_link(label, cid)
    }

    fn read_link(&self, label: &str) -> Result<Option<Cid>> {
        sleep(Duration::from_millis(1));
        self.0.read_link(label)
    }

    fn remove_link(&self, label: &str) -> Result<()> {
        sleep(Duration::from_millis(1));
        self.0.remove_link(label)
    }
}

type StoreSetup = Box<dyn Fn() -> Box<dyn Store>>;

fn store_bench(c: &mut Criterion, stores: Vec<(&str, StoreSetup)>) {
    let blocks = [gen_block(0)];

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} read:miss", store_name), |b| {
            let store = store_setup();
            let (cid, _) = &blocks[0];
            b.iter(|| {
                black_box(store.read(black_box(cid))).unwrap();
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} read:after-write", store_name), |b| {
            let store = store_setup();
            let (cid, data) = &blocks[0];
            store.write(cid, data.clone()).unwrap();
            b.iter(|| {
                black_box(store.read(black_box(cid))).unwrap().unwrap();
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} read:after-flush", store_name), |b| {
            let store = store_setup();
            let (cid, data) = &blocks[0];
            store.write(cid, data.clone()).unwrap();
            store.flush().unwrap();
            b.iter(|| {
                black_box(store.read(black_box(cid))).unwrap().unwrap();
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} write:exists", store_name), |b| {
            let store = store_setup();
            let (cid, data) = &blocks[0];
            store.write(cid, data.clone()).unwrap();
            b.iter(|| {
                black_box(store.write(black_box(cid), black_box(data.clone()))).unwrap();
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} write-flush:exists", store_name), |b| {
            let store = store_setup();
            let (cid, data) = &blocks[0];
            store.write(cid, data.clone()).unwrap();
            store.flush().unwrap();
            b.iter(|| {
                black_box({
                    store
                        .write(black_box(cid), black_box(data.clone()))
                        .unwrap();
                    store.flush().unwrap();
                });
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} pin:pinned", store_name), |b| {
            let store = store_setup();
            let (cid, _) = &blocks[0];
            store.pin(cid).unwrap();
            b.iter(|| {
                black_box(store.pin(cid)).unwrap();
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} pin-flush:pinned", store_name), |b| {
            let store = store_setup();
            let (cid, _) = &blocks[0];
            store.pin(cid).unwrap();
            store.flush().unwrap();
            b.iter(|| {
                black_box({
                    store.pin(cid).unwrap();
                    store.flush().unwrap();
                });
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} unpin", store_name), |b| {
            let store = store_setup();
            let (cid, _) = &blocks[0];
            b.iter(|| {
                black_box(store.unpin(cid)).unwrap();
            });
        });
    }

    for (store_name, store_setup) in &stores {
        c.bench_function(&format!("{} unpin-flush", store_name), |b| {
            let store = store_setup();
            let (cid, _) = &blocks[0];
            b.iter(|| {
                black_box({
                    store.unpin(cid).unwrap();
                    store.flush().unwrap();
                });
            });
        });
    }

    // TODO write/read/remove link
}

fn bench_stores(c: &mut Criterion) {
    let mem_store: StoreSetup = Box::new(|| Box::new(MemStore::default()));
    let buf_store: StoreSetup = Box::new(|| Box::new(BufStore::new(MemStore::default(), 16, 16)));
    let buf_delay_store: StoreSetup =
        Box::new(|| Box::new(BufStore::new(DelayStore::new(MemStore::default()), 16, 16)));

    let stores = vec![
        ("mem", mem_store),
        ("buf", buf_store),
        ("delay", buf_delay_store),
    ];
    store_bench(c, stores);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_stores
}

criterion_main!(benches);
