use async_std::fs::{self, File};
use async_std::io::Write;
use async_trait::async_trait;
use libipld::{Cid, Result, Store};
use multibase::Base;
use std::path::{Path, PathBuf};

pub struct BlockStore(Box<Path>);

impl BlockStore {
    #[inline]
    fn path(&self, cid: &Cid) -> Box<Path> {
        let base64 = multibase::encode(Base::Base64UrlUpperNoPad, cid.to_bytes());
        let mut buf = PathBuf::from(self.0.clone());
        buf.push(base64);
        buf.into_boxed_path()
    }
}

#[async_trait]
impl Store for BlockStore {
    fn new(path: Box<Path>) -> Self {
        Self(path)
    }

    async fn read(&self, cid: &Cid) -> Result<Box<[u8]>> {
        let path = self.path(cid);
        let bytes = fs::read(path).await?;
        Ok(bytes.into_boxed_slice())
    }

    async fn write(&self, cid: &Cid, data: &Box<[u8]>) -> Result<()> {
        let path = self.path(cid);
        // Only write if file doesn't exist.
        if fs::metadata(&path).await.is_ok() {
            return Ok(());
        }
        let mut file = File::create(&path).await?;
        file.write_all(data).await?;
        //file.sync_data().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use tempdir::TempDir;

    #[test]
    fn store_works() {
        task::block_on(async {
            let tmp = TempDir::new("store").unwrap();
            let store = BlockStore::new(tmp.path().into());
            let cid = Cid::random();
            let data = vec![0, 1, 2, 3].into_boxed_slice();
            store.write(&cid, &data).await.unwrap();
            let data2 = store.read(&cid).await.unwrap();
            assert_eq!(data, data2);
            tmp.close().unwrap();
        });
    }
}
