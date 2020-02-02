// Link
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_link {
    ($name:ident $type:ty) => {
        struct $name(::libipld_schema::Link<$type>);
        // type $name = Link<$type>;
    };
}

// List
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_list {
    // ($name:ident : $type:ty) => {
    //     typedef_list!($name $type);
    // };
    ($name:tt $elem_type:ty) => {
        struct $name(::std::vec::Vec<$elem_type>);
        // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
        delegate_repr_impl!($name: (::std::vec::Vec<$elem_type>));
    };
}

//////////////////////////////////////////////////////////////////////////
// Map
//////////////////////////////////////////////////////////////////////////
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_map {
    // basic map representation
    ($name:ident { $key:ty : $value:ty }) => {
        struct $name(::std::collections::BTreeMap<$key, $value>);
        // TODO: assert (with a trait-if macro) that the key impls Ord,
        if !::impls::impls!(
            $key: ::std::cmp::Ord
                & ::libipld_schema::Representation
                & Into<::libipld::cbor::ipld::IpldIndex>
        ) {
            panic!(
                "\
                 {} can only be constructed with key types that implement \
                 `std::cmp::Ord` and `libipld_schema::Representation`\
                 ",
                stringify!($name)
            );
        }
        delegate_repr_impl!($name: (::$std::collections::BTreeMap<$key, $value>));
    };
    // map stringpairs representation
    ($name:ident { $key:ty : $value:ty } @stringpairs $inner:expr, $entry:expr) => {
        struct $name(::std::collections::BTreeMap<$key, $value>);
        // repr_map_impl_stringpairs!($name { $key : $value } { $inner, $entry });
    };
    // map listpairs representation
    ($name:ident { $key:ty : $value:ty } @listpairs) => {
        struct $name(::std::collections::BTreeMap<$key, $value>);
        // repr_map_impl_listpairs!($name { $key : $value });
    };
}

// // Delegate representation
// // delegates to the inner type's `Representation` implementation
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! delegate_recursive_repr_impl {
//     // delegation impl
//     ($name:tt : $type:tt) => {
//         #[::libipld_schema::prelude::async_trait]
//         impl<R, W> ::libipld_schema::Representation<R, W> for $name
//         where
//             R: ::libipld_schema::prelude::Read + Unpin + Send,
//             W: ::libipld_schema::prelude::Write + Unpin + Send,
//         {
//             #[inline]
//             async fn read<C>(ctx: &mut C) -> Result<Self, ::libipld_schema::Error>
//             where
//                 R: 'async_trait,
//                 W: 'async_trait,
//                 C: ::libipld_schema::Context<R, W> + Send,
//             {
//                 Ok($name(<$type>::read(ctx).await?))
//             }

//             #[inline]
//             async fn write<C>(&self, ctx: &mut C) -> Result<(), ::libipld_schema::Error>
//             where
//                 R: 'async_trait,
//                 W: 'async_trait,
//                 C: ::libipld_schema::Context<R, W> + Send,
//             {
//                 <$type>::write(&self.0, ctx).await
//             }
//         }
//     };
// }
