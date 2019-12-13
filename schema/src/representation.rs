use async_trait::async_trait;
use libipld::{
    cbor::{decode::Read, encode::Write, CborError, ReadCbor, WriteCbor},
    cid::Cid,
    error::BlockError,
};

///
#[async_trait]
pub trait BlockContext<R: Read, W: Write> {
    ///
    fn reader(&mut self, cid: &Cid) -> &mut R;

    ///
    fn writer(&mut self) -> &mut W;

    ///
    async fn flush(&self) -> Result<Cid, BlockError>;
}

///
#[async_trait]
pub trait Representation<R, W, C>: ReadCbor + WriteCbor
where
    R: Read,
    W: Write,
    C: BlockContext<R, W>,
{
    ///
    type Context = C;

    ////
    async fn read(cid: &Cid, ctx: &mut Self::Context) -> Result<Self, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        Self::Context: 'async_trait;

    ///
    async fn write(&self, ctx: &mut Self::Context) -> Result<Cid, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        Self::Context: 'async_trait;
}

#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for T
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: BlockContext<R, W> + Send,
    T: ReadCbor + WriteCbor + Sync,
{
    ///
    type Context = C;

    ///
    default async fn read(cid: &Cid, ctx: &mut Self::Context) -> Result<Self, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        Self::Context: 'async_trait,
    {
        Self::read_cbor(ctx.reader(cid)).await
    }

    ///
    default async fn write(&self, ctx: &mut Self::Context) -> Result<Cid, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        Self::Context: 'async_trait,
    {
        self.write_cbor(ctx.writer()).await?;
        Ok(Cid::random())
    }
}
