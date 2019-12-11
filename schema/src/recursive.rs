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
        #[derive(DagCbor)]
        pub struct $name(Vec<$elem_type>);
    };
}

// Map
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_map {
    ($name:ident {$key:ty : $value:ty}) => {
        #[derive(DagCbor)]
        pub struct $name(std::collections::BTreeMap<$key, $value>);

        schema_repr_map!();
    };
    ($name:ident {$key:ty : $value:ty} $repr:tt*) => {
        #[derive(DagCbor)]
        pub struct $name(std::collections::BTreeMap<$key, $value>);

        schema_repr_map!($repr);
    };
}

// Struct
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_struct {
    ($name:ident {}) => {
        #[derive(DagCbor)]
        pub struct $name;
    };
}

//////////////////////////////////////////////////////////////////////////
// Representation Impls
//////////////////////////////////////////////////////////////////////////

// Map representations
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_map {
    () => {};

    ($name:ident { $key:ty : $value:ty } { $inner:ident : $entry:ident }) => {};

    ($name:ident { $key:ty : $value:ty } listpairs) => {};
}
