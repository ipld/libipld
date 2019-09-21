//! Ipld dag.
use crate::error::{BlockError, DagError};
use crate::hash::Hash;
use crate::ipld::{Cid, Ipld};
use crate::path::Path;
use crate::store::{BlockStore, Cache, Store};

/// Path in a dag.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct DagPath<'a>(&'a Cid, Path);

impl<'a> DagPath<'a> {
    /// Create a new dag path.
    pub fn new<T: Into<Path>>(cid: &'a Cid, path: T) -> Self {
        Self(cid, path.into())
    }
}

impl<'a> From<&'a Cid> for DagPath<'a> {
    fn from(cid: &'a Cid) -> Self {
        Self(cid, Default::default())
    }
}

/// The DAG.
pub struct Dag<TStore: Store, TCache: Cache> {
    store: BlockStore<TStore, TCache>,
}

impl<TStore: Store, TCache: Cache> Dag<TStore, TCache> {
    /// Creates a new Dag.
    pub fn new(store: TStore, cache: TCache) -> Self {
        Self {
            store: BlockStore::new(store, cache),
        }
    }

    /// Retrives a block from the store.
    pub async fn get_ipld(&mut self, cid: &Cid) -> Result<Option<Ipld>, BlockError> {
        self.store.read_ipld(cid).await
    }

    /// Retrives ipld from the dag.
    pub async fn get<'a>(&mut self, path: &DagPath<'a>) -> Result<Option<Ipld>, DagError> {
        let root = self.store.read_ipld(&path.0).await?;
        let mut root = if let Some(root) = root {
            root
        } else {
            return Ok(None);
        };
        let mut ipld = &root;
        for segment in path.1.iter() {
            if let Some(next) = match ipld {
                Ipld::List(_) => {
                    let index: usize = segment.parse()?;
                    ipld.get(index)
                }
                Ipld::Map(_) => ipld.get(segment.as_str()),
                _ => return Err(DagError::NotIndexable),
            } {
                if let Ipld::Link(cid) = next {
                    if let Some(cid) = self.store.read_ipld(cid).await? {
                        root = cid;
                        ipld = &root;
                    } else {
                        return Ok(None);
                    }
                } else {
                    ipld = next;
                }
            } else {
                return Ok(None);
            }
        }
        Ok(Some(ipld.to_owned()))
    }

    /// Puts ipld into the dag.
    pub async fn put_ipld<H: Hash>(&mut self, ipld: &Ipld) -> Result<Cid, BlockError> {
        let cid = self.store.write_cbor::<H, _>(ipld).await?;
        self.store.flush().await?;
        Ok(cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Blake2b;
    use crate::ipld;
    use crate::store::mock::{MemCache, MemStore};
    use async_std::task;

    #[test]
    fn test_dag() {
        task::block_on(async {
            let store = MemStore::default();
            let cache = MemCache::default();
            let mut dag = Dag::<MemStore, MemCache>::new(store, cache);
            let ipld1 = ipld!({"a": 3});
            let cid = dag.put_ipld::<Blake2b>(&ipld1).await.unwrap();
            let ipld2 = ipld!({"root": [{"child": &cid}]});
            let root = dag.put_ipld::<Blake2b>(&ipld2).await.unwrap();
            let path = DagPath::new(&root, "root/0/child/a");
            assert_eq!(dag.get(&path).await.unwrap(), Some(Ipld::Integer(3)));
        });
    }
}
