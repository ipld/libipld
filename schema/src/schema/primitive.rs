// Null
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_null {
    ($name:ident) => {
        type $name = ();
    };
}

// Bool
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_bool {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(bool);
        schema_repr_delegate!($name: bool);
    };
}

// Int, Float
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_num {
    ($name:ident $type:ty) => {
        #[derive(Debug)]
        struct $name($type);
        // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
        // schema_repr_delegate!($name: $type);
    };
}

// String
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_str {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(String);
        schema_repr_delegate!($name: String);
    };
}

// Bytes
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_bytes {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(Bytes);
        schema_repr_delegate!($name: Bytes);
    };
}

//////////////////////////////////////////////////////////////////////////
// Representation Impls
//////////////////////////////////////////////////////////////////////////

// Delegate representation
// delegates to the inner type's `Representation` implementation
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_delegate {
    ($name:tt : ($type:tt)) => {
        schema_repr_delegate!($name: $type);
    };

    ($name:tt : $type:tt) => {
        #[async_trait]
        impl<R, W> Representation<R, W> for $name
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
                Ok($name(<$type>::read(ctx).await?))
            }

            #[inline]
            async fn write<C>(&self, ctx: &mut C) -> Result<(), Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: Context<R, W> + Send,
            {
                <$type>::write(&self.0, ctx).await
            }
        }
    };
}
