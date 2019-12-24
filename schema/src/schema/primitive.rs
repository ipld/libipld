// Null
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_null {
    ($name:ident) => {
        type $name = ();
    };
}

// Bool
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_bool {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(bool);
        schema_repr_delegate!($name: bool);
    };
}

// Int, Float
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_num {
    ($name:ident $type:ty) => {
        #[derive(Debug)]
        struct $name($type);
        schema_repr_num!($name $type);
    };
}

// String
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_str {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(String);
        schema_repr_delegate!($name: String);
    };
}

// Bytes
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
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_delegate {
    ($name:tt : $type:tt) => {
        #[async_trait]
        impl<R, W, C> Representation<R, W, C> for $name
        where
            R: Read + Unpin + Send,
            W: Write + Unpin + Send,
            C: ReadContext<R> + WriteContext<W> + Send,
        {
            #[inline]
            async fn read(ctx: &mut C) -> Result<Self, Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: 'async_trait,
            {
                Ok($name(<$type>::read(ctx).await?))
            }

            #[inline]
            async fn write(&self, ctx: &mut C) -> Result<(), Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: 'async_trait,
            {
                <$type>::write(&self.0, ctx).await
            }
        }
    };
}

// Int representations
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_num {
    ($name:ident $type:ty) => {};
}
