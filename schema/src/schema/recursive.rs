// Link
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_link {
    ($name:ident $type:ty) => {
        type $name = Link<$type>;
    };
}

// List
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_list {
    ($name:ident $elem_type:ty) => {
        #[derive(Debug)]
        struct $name(Vec<$elem_type>);
        // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
        // schema_repr_delegate!($name: (Vec<$elem_type>));
    };
}

// Map
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_map {
    // normal representation
    ($name:ident { $key:ty : $value:ty }) => {
        #[derive(Debug)]
        struct $name(BTreeMap<$key, $value>);
        // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
        // schema_repr_delegate!($name: (BTreeMap<$key, $value>));
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
#[doc(hidden)]
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

// stringpairs
#[doc(hidden)]
#[macro_export(local_inner_macros)]
// TODO: impl ToString for the type, and require that it's member's implement it
macro_rules! schema_repr_map_impl_stringpairs {
    ($name:tt { $key:tt : $value:tt } { $inner:tt, $entry:tt }) => {};
}

// listpairs
// TODO:
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_map_impl_listpairs {
    ($name:tt { $key:tt : $value:tt }) => {};
}
