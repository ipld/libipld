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
        #[cfg_attr(feature = "graphql", derive(juniper::GraphQLScalarValue))]
        struct $name(bool);
        delegate_repr_impl!($name: bool);
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
        delegate_repr_impl!($name: $type);
    };
}

// String
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_str {
    ($name:ident) => {
        #[cfg_attr(feature = "graphql", derive(juniper::GraphQLScalarValue))]
        struct $name(String);
        delegate_repr_impl!($name: String);
    };
}

// Bytes
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_bytes {
    ($name:ident) => {
        struct $name(::libipld_schema::prelude::Bytes);
        delegate_repr_impl!($name : (::libipld_schema::prelude::Bytes));

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

//////////////////////////////////////////////////////////////////////////
// Representation Impls
//////////////////////////////////////////////////////////////////////////

// Delegate representation
// delegates to the inner type's `Representation` implementation
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! delegate_repr_impl {
    ($name:tt : ($type:tt)) => {
        delegate_repr_impl!($name: $type);
    };

    // delegation impl
    ($name:tt : $type:tt) => {
        #[::libipld_schema::prelude::async_trait]
        impl<R, W> ::libipld_schema::Representation<R, W> for $name
        where
            R: ::libipld_schema::prelude::Read + Unpin + Send,
            W: ::libipld_schema::prelude::Write + Unpin + Send,
        {
            #[inline]
            async fn read<C>(ctx: &mut C) -> Result<Self, ::libipld_schema::Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: ::libipld_schema::Context<R, W> + Send,
            {
                Ok($name(<$type>::read(ctx).await?))
            }

            #[inline]
            async fn write<C>(&self, ctx: &mut C) -> Result<(), ::libipld_schema::Error>
            where
                R: 'async_trait,
                W: 'async_trait,
                C: ::libipld_schema::Context<R, W> + Send,
            {
                <$type>::write(&self.0, ctx).await
            }
        }
    };
}
