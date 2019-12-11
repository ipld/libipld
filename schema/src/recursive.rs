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
        struct $name(std::collections::BTreeMap<$key, $value>);
        schema_repr_delegate!($name: (BTreeMap<$key, $value>));
    };
    ($name:ident { $key:ty : $value:ty } { $inner:expr, $entry:expr }) => {
        #[derive(Debug)]
        struct $name(std::collections::BTreeMap<$key, $value>);
        schema_repr_map_stringpairs!($name { $key : $value } { $inner, $entry });
    };
    ($name:ident { $key:ty : $value:ty } @listpairs) => {
        #[derive(Debug)]
        struct $name(std::collections::BTreeMap<$key, $value>);
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
    ($name:ident { $key:ty : $value:ty } { $inner:expr, $entry:expr }) => {};
}
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_map_listpairs {
    // listpairs
    ($name:ident { $key:ty : $value:ty }) => {};
}
