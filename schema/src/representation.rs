use async_trait::async_trait;
use libipld::{
    cbor::{decode::Read, encode::Write, CborError, ReadCbor, WriteCbor},
    cid::Cid,
    error::BlockError,
};

#[async_trait]
pub trait Context<R: Read, W: Write + Unpin + Send> {
    fn reader(&mut self, cid: &Cid) -> &mut R;

    fn writer(&mut self) -> &mut W;

    async fn flush(&self) -> Result<Cid, BlockError>;
}

///
#[async_trait]
pub trait Representation<R: Read + Unpin + Send, W: Write + Unpin + Send>:
    ReadCbor + WriteCbor
{
    type Context: Context<R, W>;

    async fn read(cid: &Cid, ctx: &mut Self::Context) -> Result<Self, CborError>;

    async fn write(&self, ctx: &mut Self::Context) -> Result<Cid, CborError>;
}

#[async_trait]
impl<R, W, C, T> Representation<R, W> for T
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: Context<R, W>,
    T: ReadCbor + WriteCbor,
{
    type Context = C;

    default async fn read(cid: &Cid, ctx: &mut C) -> Result<Self, CborError> {
        Self::read_cbor(ctx.reader(cid)).await
    }

    default async fn write(&self, ctx: &mut C) -> Result<Cid, CborError> {
        self.write_cbor(ctx.writer()).await?;
        Ok(Cid::random())
    }
}
