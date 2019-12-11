use async_trait::async_trait;
use libipld::{
    cbor::{decode::Read, encode::Write, CborError, ReadCbor, WriteCbor},
    Cid,
};

///
#[derive(Debug)]
pub enum Link<T> {
    ///
    Cid(Cid),
    ///
    Dag(T),
}

#[async_trait]
impl<T: WriteCbor + Sync> WriteCbor for Link<T> {
    async fn write_cbor<W: Write + Unpin + Send>(&self, w: &mut W) -> Result<(), CborError> {
        match self {
            Link::Cid(cid) => cid.write_cbor(w).await,
            Link::Dag(t) => t.write_cbor(w).await,
        }
    }
}

#[async_trait]
impl<T: ReadCbor + Send> ReadCbor for Link<T> {
    async fn try_read_cbor<R: Read + Unpin + Send>(
        r: &mut R,
        major: u8,
    ) -> Result<Option<Self>, CborError> {
        Ok(None)
    }
}
