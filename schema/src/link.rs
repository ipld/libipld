use crate::{async_trait, CborError, ReadCbor, WriteCbor};
use crate::{BlockReadContext, BlockWriteContext, Cid, Error, Read, Representation, Write};

/// Link type, used to switch between a `Cid` and it's underlying dag.
#[derive(Debug)]
pub enum Link<T> {
    /// Represents a raw `Cid` contained within a dag.
    Cid(Cid),

    /// Represents a raw `Cid` and an instance of the type it represents.
    Dag(Cid, T),
}

#[async_trait]
impl<R, W, C, T> Representation<R, W, C> for Link<T>
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
    C: BlockReadContext<R> + BlockWriteContext<W> + Send,
    T: Representation<R, W, C> + Sync,
{
    #[inline]
    async fn read(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        let cid = Cid::read(ctx).await?;
        if ctx.should_read(&cid) {
            let dag = T::read(ctx).await?;
            Ok(Link::Dag(cid, dag))
        } else {
            Ok(Link::Cid(cid))
        }
    }

    #[inline]
    async fn write(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: 'async_trait,
    {
        match self {
            Link::Cid(cid) => {
                Cid::write(cid, ctx).await?;
                Ok(())
            }
            Link::Dag(old_cid, dag) => {
                if ctx.start() {
                    T::write(dag, ctx).await?;
                    let cid = ctx.end(Some(old_cid)).await?;
                    Cid::write(&cid, ctx).await?;
                } else {
                    Cid::write(old_cid, ctx).await?;
                }
                Ok(())
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// additional implementations
////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl<T: WriteCbor + Sync> WriteCbor for Link<T> {
    async fn write_cbor<W: Write + Unpin + Send>(&self, _w: &mut W) -> Result<(), CborError> {
        unimplemented!()
    }
}

#[async_trait]
impl<T: ReadCbor + Send> ReadCbor for Link<T> {
    async fn try_read_cbor<R: Read + Unpin + Send>(
        _r: &mut R,
        _major: u8,
    ) -> Result<Option<Self>, CborError> {
        unimplemented!()
    }
}

impl<T> From<Cid> for Link<T> {
    fn from(cid: Cid) -> Self {
        Link::Cid(cid)
    }
}
