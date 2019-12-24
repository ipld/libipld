use crate::{async_trait, ReadCbor, WriteCbor};
use crate::{Bytes, Cid, Context, Error, Read, Representation, Write};

/// A default blanket overridable implementation that delegates directly to the underlying `ReadCbor`/`WriteCbor`. Should generally only need to be overwritten for recursive types or types not defined with the `schema!` macro.
macro_rules! primitive_representation_impl {
    ($type:tt) => {
        #[async_trait]
        impl<R, W> Representation<R, W> for $type
        where
            R: Read + Unpin + Send,
            W: Write + Unpin + Send,
        {
            #[inline]
            async fn read<C>(ctx: &mut C) -> Result<Self, Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: Context<R, W> + Send,
            {
                let t = <$type>::read_cbor(ctx.reader()).await?;
                Ok(t)
            }

            #[inline]
            async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: Context<R, W> + Send,
            {
                <$type>::write_cbor(self, ctx.writer()).await?;
                Ok(())
            }
        }
    };
}

primitive_representation_impl!(());
primitive_representation_impl!(bool);
primitive_representation_impl!(u8);
primitive_representation_impl!(u16);
primitive_representation_impl!(u32);
primitive_representation_impl!(u64);
primitive_representation_impl!(i8);
primitive_representation_impl!(i16);
primitive_representation_impl!(i32);
primitive_representation_impl!(i64);
primitive_representation_impl!(f32);
primitive_representation_impl!(f64);
primitive_representation_impl!(String);
primitive_representation_impl!(Cid);

#[async_trait]
impl<R, W> Representation<R, W> for Bytes
where
    R: Read + Unpin + Send,
    W: Write + Unpin + Send,
{
    #[inline]
    async fn read<C>(ctx: &mut C) -> Result<Self, Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
    {
        let bytes = <Box<[u8]>>::read_cbor(ctx.reader()).await?;
        Ok(Bytes::copy_from_slice(bytes.as_ref()))
    }

    #[inline]
    async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
    where
        R: 'async_trait,
        W: 'async_trait,
        C: Context<R, W> + Send,
    {
        <[u8]>::write_cbor(self, ctx.writer()).await?;
        Ok(())
    }
}
