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
    };
}

// Int, Float
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_num {
    ($name:ident $type:ty) => {
        #[derive(Debug)]
        struct $name($type);
        schema_repr_i8!($type);
    };
}

// String
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_str {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(String);
    };
}

// Bytes
#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_bytes {
    ($name:ident) => {
        #[derive(Debug)]
        struct $name(Vec<u8>);
    };
}

//////////////////////////////////////////////////////////////////////////
// Representation Impls
//////////////////////////////////////////////////////////////////////////

// Int representations
#[macro_export(local_inner_macros)]
macro_rules! schema_repr_i8 {
    ($type:ty) => {};
}
