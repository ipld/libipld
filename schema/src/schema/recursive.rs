// Link
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_link {
    ($name:ident $type:ty) => {
        type $name = Link<$type>;
    };
}

// List
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_list {
    ($name:ident $elem_type:ty) => {
        #[derive(Debug)]
        struct $name(Vec<$elem_type>);
        schema_repr_delegate_recursive!($name: ((Vec<$elem_type>)));
    };
}

// Map
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_map {
    // normal representation
    ($name:ident { $key:ty : $value:ty }) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        schema_repr_delegate_recursive!($name: ((BTreeMap<$key, $value>)));
    };
    // stringpairs
    ($name:ident { $key:ty : $value:ty } { $inner:expr, $entry:expr }) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        schema_repr_map_impl_stringpairs!($name { $key : $value } { $inner, $entry });
    };
    // listpairs
    ($name:ident { $key:ty : $value:ty } @listpairs) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        schema_repr_map_impl_listpairs!($name { $key : $value });
    };
}

// Struct
// TODO:
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_struct {
    ($name:ident {}) => {
        #[derive(Debug)]
        pub struct $name;
    };
}

//////////////////////////////////////////////////////////////////////////
// Representation Impls
//////////////////////////////////////////////////////////////////////////

// TODO: get rid of this since context constraints arent working
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_delegate_recursive {
    ($name:tt : (($type:tt))) => {
        schema_repr_delegate_recursive!($name: ($type))
    };

    // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
    ($name:tt : ($type:tt)) => {
        #[async_trait]
        impl<R, W, C> Representation<R, W, C> for $name
        where
            R: Read + Unpin + Send,
            W: Write + Unpin + Send,
            C: ReadContext<R> + WriteContext<W> + RecursiveContext + Send,
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

// stringpairs
#[macro_export(local_inner_macros)]
// TODO: impl ToString for the type, and require that it's member's implement it
macro_rules! schema_repr_map_impl_stringpairs {
    ($name:tt { $key:tt : $value:tt } { $inner:tt, $entry:tt }) => {};
}

// listpairs
// TODO:
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_map_impl_listpairs {
    ($name:tt { $key:tt : $value:tt }) => {};
}
