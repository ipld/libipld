// Link
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_link {
    ($name:ident $type:ty) => {
        type $name = Link<$type>;
    };
}

// List
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_list {
    ($name:ident $elem_type:ty) => {
        struct $name(Vec<$elem_type>);
        // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
        // delegate_repr_impl!($name: (Vec<$elem_type>));
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
        struct $name(BTreeMap<$key, $value>);
        // TODO: fix matching against `tt`: https://github.com/dtolnay/async-trait/issues/46#issuecomment-547572251
        // delegate_repr_impl!($name: (BTreeMap<$key, $value>));
    };
    // map stringpairs representation
    ($name:ident { $key:ty : $value:ty } @stringpairs $inner:expr, $entry:expr) => {
        struct $name(BTreeMap<$key, $value>);
        // repr_map_impl_stringpairs!($name { $key : $value } { $inner, $entry });
    };
    // map listpairs representation
    ($name:ident { $key:ty : $value:ty } @listpairs) => {
        struct $name(BTreeMap<$key, $value>);
        // repr_map_impl_listpairs!($name { $key : $value });
    };
}

//////////////////////////////////////////////////////////////////////////
// Struct
//////////////////////////////////////////////////////////////////////////
// TODO:
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_struct {
    // struct map representation
    ($name:ident { /* fields */ }) => {};
    // struct tuple representation
    ($name:ident { /* fields */ }) => {};
    // struct stringpairs representation
    ($name:ident { /* fields */ }) => {};
    // struct stringjoin representation
    ($name:ident { /* fields */ }) => {};
    // struct listpairs representation
    ($name:ident { /* fields */ }) => {};
}

//////////////////////////////////////////////////////////////////////////
// Enum
//////////////////////////////////////////////////////////////////////////
// TODO:
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_enum {
    // enum string representation
    ($name:ident { /* fields */ }) => {};
    // enum int representation
    ($name:ident { /* fields */ }) => {};
}

//////////////////////////////////////////////////////////////////////////
// Union
//////////////////////////////////////////////////////////////////////////
// TODO:
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef_union {
    // union keyed representation
    ($name:ident { /* fields */ }) => {};
    // union kinded representation
    ($name:ident { /* fields */ }) => {};
    // union envelope representation
    ($name:ident { /* fields */ }) => {};
    // union inline representation
    ($name:ident { /* fields */ }) => {};
    // union byteprefix representation
    ($name:ident { /* fields */ }) => {};
}
