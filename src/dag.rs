//! Ipld dag.
use crate::error::{format_err, Result};
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
    pub fn new(cache_size: usize) -> Self {
        Self {
            store: BlockStore::new(cache_size),
        }
    }

    /// Retrives a block from the store.
    pub fn get_ipld(&mut self, cid: &Cid) -> Result<Ipld> {
        self.store.read_ipld(cid)
    }

    /// Retrives ipld from the dag.
    pub fn get(&mut self, path: &DagPath) -> Result<Option<Ipld>> {
        let mut root = self.store.read_ipld(&path.0)?;
        let mut ipld = &root;
        for segment in path.1.iter() {
            if let Some(next) = match ipld {
                Ipld::List(_) => {
                    let index: usize = segment.parse()?;
                    ipld.get(index)
                }
                Ipld::Map(_) => ipld.get(segment.as_str()),
                _ => return Err(format_err!("Cannot index into {:?}", ipld)),
            } {
                if let Ipld::Link(cid) = next {
                    root = self.store.read_ipld(cid)?;
                    ipld = &root;
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
    pub fn put_ipld<H: Hash>(&mut self, ipld: &Ipld) -> Result<Cid> {
        self.store.write_cbor::<H, _>(ipld)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Blake2b;
    use crate::ipld;
    use crate::store::mock::{MemCache, MemStore};

    #[test]
    fn test_dag() {
        let mut dag = Dag::<MemStore, MemCache>::new(16);
        let cid = dag.put_ipld::<Blake2b>(&ipld!({"a": 3})).unwrap();
        let root = dag
            .put_ipld::<Blake2b>(&ipld!({"root": [{"child": &cid}]}))
            .unwrap();
        let path = DagPath::new(&root, "root/0/child/a");
        assert_eq!(dag.get(&path).unwrap(), Some(Ipld::Integer(3)));
    }
}
