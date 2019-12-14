use super::{BlockContext, Context, RecursiveContext};
use crate::{async_trait, CborError, Cid, Read, ReadCbor, Write, WriteCbor};

pub mod context;

///
#[async_trait]
pub trait Representation<R, W, C>: ReadCbor + WriteCbor
where
    R: Read,
    W: Write,
    C: Context<R, W>,
{
    ////
    async fn read(ctx: &mut C) -> Result<Self, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait;

    ///
    async fn write(&self, ctx: &mut C) -> Result<(), CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait;
}

#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for T
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: Context<R, W> + Send,
    T: ReadCbor + WriteCbor + Sync,
{
    ///
    default async fn read(ctx: &mut C) -> Result<Self, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        Self::read_cbor(ctx.reader()).await
    }

    ///
    default async fn write(&self, ctx: &mut C) -> Result<(), CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        self.write_cbor(ctx.writer()).await
    }
}

#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for Option<T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: RecursiveContext<R, W> + Send,
    T: ReadCbor + WriteCbor + Sync,
{
    ///
    async fn read(ctx: &mut C) -> Result<Self, CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        match u8::read(ctx).await? {
            0xf6 => Ok(None),
            0xf7 => Ok(None),
            // TODO: is this right?
            _ => match T::read(ctx).await {
                Err(CborError::UnexpectedCode) => Ok(None),
                Err(err) => Err(err),
                Ok(t) => Ok(Some(t)),
            },
        }
    }

    ///
    async fn write(&self, ctx: &mut C) -> Result<(), CborError>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        if let Some(value) = self {
            value.write(ctx).await
        } else {
            self.write_cbor(ctx.writer()).await
        }
    }
}

