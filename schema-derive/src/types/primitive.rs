// Null
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_null {
    ($name:ident) => {
        type $name = ();
    };
}

// Bool
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_bool {
    ($name:ident) => {
        type $name = bool;
    };
}

// Int, Float
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_num {
    ($name:ident : $type:tt) => {
        // TODO: GraphQL nums are fixed to i32 and f64
        #[cfg_attr(feature = "graphql", derive(juniper::GraphQLScalarValue))]
        struct $name($type);
        // delegate_repr_impl!($name: $type);
    };
}

// String
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_str {
    ($name:ident) => {
        #[cfg_attr(feature = "graphql", derive(juniper::GraphQLScalarValue))]
        struct $name(String);
        // delegate_repr_impl!($name: String);

        // impl<'a> Into<::libipld_schema::dev::IpldIndex<'a>> for $name {
        //     fn into(self) -> ::libipld_schema::dev::IpldIndex<'a> {
        //         ::libipld_schema::dev::IpldIndex::from(self.0)
        //     }
        // }
    };
}

// Bytes
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_bytes {
    ($name:ident) => {
        // struct $name(::libipld_schema::dev::Bytes);
        // delegate_repr_impl!($name: (::libipld_schema::dev::Bytes));

        // #[cfg(feature = "graphql")]
        // juniper::graphql_scalar!($name {
        //     description: ""
        //     resolve(&self) -> juniper::Value {
        //         juniper::Value::string(&self.0)
        //     }

        //     from_input_value(v: &juniper::InputValue) -> Option<$name> {
        //         v.as_string_value().map(|s| $name(s.to_owned()))
        //     }

        //     from_str<'a>(value: juniper::ScalarToken<'a>) -> juniper::ParseScalarResult<'a> {
        //         <String as juniper::ParseScalarValue>::from_str(value)
        //     }
        // });
    };
}

// //////////////////////////////////////////////////////////////////////////
// // Representation Impls
// //////////////////////////////////////////////////////////////////////////

// // Delegate representation
// // delegates to the inner type's `Representation` implementation
// #[doc(hidden)]
// #[macro_export(local_inner_macros)]
// macro_rules! delegate_repr_impl {
//     ($name:tt : ($type:tt)) => {
//         delegate_repr_impl!($name: $type);
//     };

//     // delegation impl
//     ($name:tt : $type:tt) => {
//         #[::libipld_schema::dev::async_trait]
//         impl<Ctx, Co, R, W> ::libipld_schema::Representation<Ctx, Co, R, W> for $name
//         where
//             Co: ::libipld_schema::dev::CodecExt<Self> + ::libipld_schema::dev::CodecExt<$type>,
//             R: ::libipld_schema::dev::Read + ::libipld_schema::dev::Seek + Unpin + Send,
//             W: ::libipld_schema::dev::Write + ::libipld_schema::dev::Seek + Unpin + Send,
//             Ctx: ::libipld_schema::Context<Co, R, W> + Send + Sync,
//         {
//             #[inline]
//             async fn read(ctx: &Ctx) -> Result<Self, ::libipld_schema::Error>
//             where
//                 Co: 'async_trait,
//                 R: 'async_trait,
//                 W: 'async_trait,
//             {
//                 Ok($name(<$type>::read(ctx).await?))
//             }

//             #[inline]
//             async fn write(&self, ctx: &Ctx) -> Result<(), ::libipld_schema::Error>
//             where
//                 Co: 'async_trait,
//                 R: 'async_trait,
//                 W: 'async_trait,
//             {
//                 <$type>::write(&self.0, ctx).await
//             }
//         }
//     };
// }
