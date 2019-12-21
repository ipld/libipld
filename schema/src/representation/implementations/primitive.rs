use crate::{async_trait, ReadCbor, WriteCbor};
use crate::{Cid, Error, Read, ReadContext, Representation, Write, WriteContext};

/// A default blanket overridable implementation that delegates directly to the underlying `ReadCbor`/`WriteCbor`. Should generally only need to be overwritten for recursive types or types not defined with the `schema!` macro.
macro_rules! primitive_representation_impl {
    ($type:tt) => {
        #[async_trait]
        impl<R, W, C> Representation<R, W, C> for $type
        where
            R: Read + Unpin + Send,
            W: Write + Unpin + Send,
            C: ReadContext<R> + WriteContext<W> + Send,
        {
            type Repr = Self;

            #[inline]
            async fn read(ctx: &mut C) -> Result<Self, Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: 'async_trait,
            {
                let t = Self::read_cbor(ctx.reader()).await?;
                Ok(t)
            }

            #[inline]
            async fn write(&self, ctx: &mut C) -> Result<(), Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: 'async_trait,
            {
                self.write_cbor(ctx.writer()).await?;
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
