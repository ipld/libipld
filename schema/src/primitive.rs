// Null
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_null {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name;
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
        struct $name(Vec<u8>);
        schema_repr_delegate!($name: (Vec<u8>));
    };
}

//////////////////////////////////////////////////////////////////////////
// Representation Impls
//////////////////////////////////////////////////////////////////////////

// Delegate representation
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_delegate {
    ($name:tt : ($type:tt)) => {
        schema_repr_delegate!($name: $type);
    };

    // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
    ($name:tt : $type:tt) => {
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

        #[async_trait]
        impl cbor::decode::ReadCbor for $name {
            async fn try_read_cbor<R: cbor::decode::Read + Unpin + Send>(
                r: &mut R,
                major: u8,
            ) -> Result<Option<Self>, cbor::CborError> {
                match <$type>::try_read_cbor(r, major).await? {
                    Some(inner) => Ok(Some($name(inner))),
                    None => Ok(None),
                }
            }

            #[inline]
            async fn read_cbor<R: cbor::decode::Read + Unpin + Send>(
                r: &mut R,
            ) -> Result<Self, cbor::CborError> {
                Ok($name(<$type>::read_cbor(r).await?))
            }
        }
    };
}

// Null representation
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_null {
    ($type:ty) => {};
}

// Int representations
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_num {
    ($name:ident $type:ty) => {};
}
