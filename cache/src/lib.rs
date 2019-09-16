use core::hash::{BuildHasher, Hasher};
use libipld::{Cache, Cid};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct BlockCache {
    cache: Mutex<HashMap<Cid, Box<[u8]>, BuildCidHasher>>,
}

impl Cache for BlockCache {
    fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::with_capacity_and_hasher(capacity, BuildCidHasher)),
        }
    }

    fn get(&self, cid: &Cid) -> Option<Box<[u8]>> {
        self.cache.lock().unwrap().get(cid).cloned()
    }

    fn put(&self, cid: Cid, data: Box<[u8]>) {
        self.cache.lock().unwrap().insert(cid, data);
    }
}

struct BuildCidHasher;

impl BuildHasher for BuildCidHasher {
    type Hasher = CidHasher;

    fn build_hasher(&self) -> Self::Hasher {
        CidHasher(None)
    }
}

struct CidHasher(Option<u64>);

impl Hasher for CidHasher {
    fn finish(&self) -> u64 {
        self.0.unwrap()
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!();
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = Some(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_works() {
        let cid1 = Cid::random();
        let data1 = vec![1].into_boxed_slice();
        let cid2 = Cid::random();
        let data2 = vec![2].into_boxed_slice();
        let cid3 = Cid::random();
        let data3 = vec![3].into_boxed_slice();

        let cache = BlockCache::new(2);
        cache.put(cid1.clone(), data1.clone());
        assert_eq!(cache.get(&cid1).as_ref(), Some(&data1));

        cache.put(cid2.clone(), data2.clone());
        assert_eq!(cache.get(&cid1).as_ref(), Some(&data1));
        assert_eq!(cache.get(&cid2).as_ref(), Some(&data2));

        cache.put(cid3.clone(), data3.clone());
        //assert_eq!(cache.get(&cid1).as_ref(), None);
        assert_eq!(cache.get(&cid2).as_ref(), Some(&data2));
        assert_eq!(cache.get(&cid3).as_ref(), Some(&data3));
    }
}
