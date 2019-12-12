use std::fmt::Display;

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
        schema_repr_delegate!($name: (Vec<$elem_type>));
    };
}

// Map
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_map {
    ($name:ident { $key:ty : $value:ty }) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        schema_repr_delegate!($name: (BTreeMap<$key, $value>));
    };
    ($name:ident { $key:ty : $value:ty } { $inner:expr, $entry:expr }) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        schema_repr_map_stringpairs!($name { $key : $value } { $inner, $entry });
    };
    ($name:ident { $key:ty : $value:ty } @listpairs) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        schema_repr_map_listpairs!($name { $key : $value });
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

// Map representations
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_map_stringpairs {
    // stringpairs
    // TODO: impl ToString for the type, and require that it's member's implement it
    ($name:tt { $key:tt : $value:tt } { $inner:tt, $entry:tt }) => {
        #[async_trait]
        impl cbor::encode::WriteCbor for $name {
            #[inline]
            async fn write_cbor<W: cbor::encode::Write + Unpin + Send>(
                &self,
                w: &mut W,
            ) -> Result<(), cbor::CborError> {
                self.0.write_cbor(w).await
            }
        }

        // #[async_trait]
        // impl cbor::decode::ReadCbor for $name {
        //     async fn try_read_cbor<R: cbor::decode::Read + Unpin + Send>(
        //         r: &mut R,
        //         major: u8,
        //     ) -> Result<Option<Self>, cbor::CborError> {
        //         match <$type>::try_read_cbor(r, major).await? {
        //             Some(inner) => Ok(Some($name(inner))),
        //             None => Ok(None),
        //         }
        //     }

        //     #[inline]
        //     async fn read_cbor<R: cbor::decode::Read + Unpin + Send>(
        //         r: &mut R,
        //     ) -> Result<Self, cbor::CborError> {
        //         Ok($name(<$type>::read_cbor(r).await?))
        //     }
        // }
    };
}
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_map_listpairs {
    // listpairs
    ($name:tt { $key:tt : $value:tt }) => {
        #[async_trait]
        impl cbor::encode::WriteCbor for $name {
            #[inline]
            async fn write_cbor<W: cbor::encode::Write + Unpin + Send>(
                &self,
                w: &mut W,
            ) -> Result<(), cbor::CborError> {
                self.0.write_cbor(w).await
            }
        }
    };
}
