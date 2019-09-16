use core::hash::{BuildHasher, Hash, Hasher};
use libipld::{Cache, Cid};
use lru::LruCache;

pub struct BlockCache(LruCache<CidHash, Box<[u8]>, BuildCidHasher>);

impl Cache for BlockCache {
    fn new(size: usize) -> Self {
        Self(LruCache::with_hasher(size, BuildCidHasher))
    }

    fn get(&mut self, cid: &Cid) -> Option<&Box<[u8]>> {
        let hash = CidHash::from(cid);
        self.0.get(&hash)
    }

    fn put(&mut self, cid: &Cid, data: Box<[u8]>) {
        let hash = CidHash::from(cid);
        self.0.put(hash, data);
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
        unimplemented!();
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = Some(i);
    }
}

#[derive(PartialEq, Eq)]
struct CidHash(u64);

impl From<&Cid> for CidHash {
    fn from(cid: &Cid) -> Self {
        let mut hash_bytes = [0u8; 8];
        let cid_bytes = cid.hash().to_bytes();
        hash_bytes.copy_from_slice(&cid_bytes[0..8]);
        CidHash(u64::from_ne_bytes(hash_bytes))
    }
}

impl Hash for CidHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
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

        let mut cache = BlockCache::new(2);
        cache.put(&cid1, data1.clone());
        assert_eq!(cache.get(&cid1), Some(&data1));

        cache.put(&cid2, data2.clone());
        assert_eq!(cache.get(&cid1), Some(&data1));
        assert_eq!(cache.get(&cid2), Some(&data2));

        cache.put(&cid3, data3.clone());
        assert_eq!(cache.get(&cid1), None);
        assert_eq!(cache.get(&cid2), Some(&data2));
        assert_eq!(cache.get(&cid3), Some(&data3));
    }
}
