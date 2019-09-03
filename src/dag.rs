//! Ipld dag.
use crate::DefaultPrefix;
use crate::block::{Block, Cid};
use crate::error::{format_err, Result};
use crate::ipld::Ipld;
use crate::path::Path;
use crate::store::IpldStore;

/// Path in a dag.
pub struct DagPath(Cid, Path);

impl DagPath {
    /// Create a new dag path.
    pub fn new<T: Into<Path>>(cid: Cid, path: T) -> Self {
        Self(cid, path.into())
    }
}

/// The DAG.
pub struct Dag<TStore: IpldStore> {
    store: TStore,
}

impl<TStore: IpldStore> Dag<TStore> {
    /// Creates a new Dag.
    pub fn new(store: TStore) -> Self {
        Self { store }
    }

    /// Retrives ipld from the dag.
    pub fn get(&self, path: &DagPath) -> Result<Ipld> {
        let mut root = self.store.read(&path.0)?;
        let mut ipld = &root;
        for segment in path.1.iter() {
            if let Some(next) = ipld.get(segment) {
                if let Ipld::Link(cid) = next {
                    root = self.store.read(cid)?;
                    ipld = &root;
                } else {
                    ipld = next;
                }
            } else {
                return Err(format_err!("Could not find {} in {:?}", segment, ipld));
            }
        }
        Ok(ipld.to_owned())
    }

    /// Puts ipld into the dag.
    pub fn put(&mut self, ipld: &Ipld) -> Result<Cid> {
        self.store.write(Block::new::<DefaultPrefix>(ipld)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipld;
    use crate::store::mock::Store;

    #[test]
    fn test_dag() {
        let store = Store::default();
        let mut dag = Dag::new(store);
        let cid = dag.put(&ipld!({"a": 3})).unwrap();
        let root = dag.put(&ipld!({"root": [{"child": &cid}]})).unwrap();
        let path = DagPath::new(root, "root/0/child/a");
        assert_eq!(dag.get(&path).unwrap(), Ipld::Integer(3));
    }
}
