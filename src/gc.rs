//! Utilities for performing garbage collection.
use crate::error::DagError;
use crate::ipld::{Cid, Ipld};
use crate::store::{BlockStore, Cache, Store};
use std::collections::HashSet;

/// Returns the references in an ipld block.
pub fn references(ipld: &Ipld) -> HashSet<Cid> {
    let mut set = HashSet::new();
    for ipld in ipld.iter() {
        if let Ipld::Link(cid) = ipld {
            set.insert(cid.to_owned());
        }
    }
    set
}

/// Returns the recursive references of an ipld block.
pub async fn closure<TStore: Store, TCache: Cache>(
    store: BlockStore<TStore, TCache>,
    roots: HashSet<Cid>,
) -> Result<HashSet<Cid>, DagError> {
    let mut stack = vec![roots];
    let mut set = HashSet::new();
    while let Some(mut roots) = stack.pop() {
        for cid in roots.drain() {
            if set.contains(&cid) {
                continue;
            }
            let ipld = store.read_ipld(&cid).await?;
            stack.push(references(&ipld));
            set.insert(cid);
        }
    }
    Ok(set)
}

/// Returns the paths to gc.
///
/// This is currently not topologically sorted according to the references
/// relationship. (p < q if q.is_reference(p))
pub async fn dead_paths<TStore: Store, TCache: Cache>(
    store: BlockStore<TStore, TCache>,
    all_cids: HashSet<Cid>,
    roots: HashSet<Cid>,
) -> Result<HashSet<Cid>, DagError> {
    let live = closure(store, roots).await?;
    let dead = all_cids.difference(&live).map(Clone::clone).collect();
    Ok(dead)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ipld, DefaultHash as H};
    use crate::store::mock::*;
    use async_std::task;

    #[test]
    fn test_references() {
        let cid1 = Cid::random();
        let cid2 = Cid::random();
        let cid3 = Cid::random();
        let ipld = ipld!({
            "cid1": &cid1,
            "cid2": { "other": true, "cid2": { "cid2": &cid2 }},
            "cid3": [[ &cid3, &cid1 ]],
        });
        let refs = references(&ipld);
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&cid1));
        assert!(refs.contains(&cid2));
        assert!(refs.contains(&cid3));
    }

    async fn run_test_closure() -> Result<(), DagError> {
        let store = BlockStore::new(MemStore::default(), MemCache::default());
        let cid1 = store.write_cbor::<H, _>(&ipld!(true)).await?;
        let cid2 = store.write_cbor::<H, _>(&ipld!({ "cid1": &cid1 })).await?;
        let cid3 = store.write_cbor::<H, _>(&ipld!([&cid2])).await?;
        let mut roots = HashSet::new();
        roots.insert(cid3.clone());
        let refs = closure(store, roots).await?;
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&cid1));
        assert!(refs.contains(&cid2));
        assert!(refs.contains(&cid3));
        Ok(())
    }

    #[test]
    fn test_closure() {
        task::block_on(run_test_closure()).unwrap();
    }
}
