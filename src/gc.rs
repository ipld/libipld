//! Utilities for performing garbage collection.
#![allow(clippy::implicit_hasher)]
use crate::block::decode_ipld;
use crate::cid::Cid;
use crate::error::Error;
use crate::ipld::Ipld;
use crate::store::ReadonlyStore;
use std::collections::HashSet;

/// Returns the references in an ipld block.
pub fn references(ipld: &Ipld) -> HashSet<Cid> {
    let mut set: HashSet<Cid> = Default::default();
    for ipld in ipld.iter() {
        if let Ipld::Link(cid) = ipld {
            set.insert(cid.to_owned());
        }
    }
    set
}

/// Returns the recursive references of an ipld block.
pub async fn closure<TStore: ReadonlyStore>(
    store: &TStore,
    roots: HashSet<Cid>,
) -> Result<HashSet<Cid>, Error> {
    let mut stack = vec![roots];
    let mut set: HashSet<Cid> = Default::default();
    while let Some(mut roots) = stack.pop() {
        for cid in roots.drain() {
            if set.contains(&cid) {
                continue;
            }
            let bytes = store.get(&cid).await?;
            let ipld = decode_ipld(&cid, &bytes)?;
            stack.push(references(&ipld));
            set.insert(cid);
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{encode, Block};
    use crate::cbor::DagCbor;
    use crate::codec::Encode;
    use crate::ipld;
    use crate::mem::MemStore;
    use crate::multihash::Sha2_256;
    use crate::store::{Store, Visibility};

    async fn insert<S: Store, E: Encode<DagCbor>>(store: &S, e: &E) -> Result<Cid, Error> {
        let Block { cid, data } = encode::<DagCbor, Sha2_256, E>(e)?;
        store.insert(&cid, data, Visibility::Public).await?;
        Ok(cid)
    }

    #[test]
    fn test_references() {
        let cid1 = Cid::new_v0(Sha2_256::digest(b"cid1")).unwrap();
        let cid2 = Cid::new_v0(Sha2_256::digest(b"cid2")).unwrap();
        let cid3 = Cid::new_v0(Sha2_256::digest(b"cid3")).unwrap();
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

    #[async_std::test]
    async fn test_closure() -> Result<(), Error> {
        let store = MemStore::default();
        let cid1 = insert(&store, &ipld!(true)).await?;
        let cid2 = insert(&store, &ipld!({ "cid1": &cid1 })).await?;
        let cid3 = insert(&store, &ipld!([&cid2])).await?;
        let mut roots: HashSet<Cid> = Default::default();
        roots.insert(cid3.clone());
        let refs = closure(&store, roots).await?;
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&cid1));
        assert!(refs.contains(&cid2));
        assert!(refs.contains(&cid3));
        Ok(())
    }
}
