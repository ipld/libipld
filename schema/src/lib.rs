//! `schema!` macro.
/// Define a native type modelling an IPLD Schema and it's Representation.
///
/// ```edition2018
/// # use libipld_schema;
/// ```
pub use libipld::*;
use link::Link;

mod advanced;
mod link;
mod recursive;

#[macro_export(local_inner_macros)]
macro_rules! schema {
    // Hide distracting implementation details from the generated rustdoc.
    ($($schema:tt)+) => {
        schema_typedef!($($schema)*);
        schema_repr!($($schema)*);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! schema_typedef {
    //////////////////////////////////////////////////////////////////////////
    // Primitive Types
    //////////////////////////////////////////////////////////////////////////

    // Null
    (type $name:ident null) => {
        #[derive(Debug)]
        pub struct $name;
    };

    // Bool
    (type $name:ident bool) => {
        #[derive(Debug)]
        pub struct $name(bool);
    };

    // Integer
    (type $name:ident i8) => {
        #[derive(Debug)]
        pub struct $name(i8);
    };
    (type $name:ident i16) => {
        #[derive(Debug)]
        pub struct $name(i16);
    };
    (type $name:ident i32) => {
        #[derive(Debug)]
        pub struct $name(i32);
    };
    (type $name:ident i64) => {
        #[derive(Debug)]
        pub struct $name(i64);
    };
    (type $name:ident u8) => {
        #[derive(Debug)]
        pub struct $name(u8);
    };
    (type $name:ident u16) => {
        #[derive(Debug)]
        pub struct $name(u16);
    };
    (type $name:ident u32) => {
        #[derive(Debug)]
        pub struct $name(u32);
    };
    (type $name:ident u64) => {
        #[derive(Debug)]
        pub struct $name(u64);
    };

    // Float
    (type $name:ident f32) => {
        #[derive(Debug)]
        pub struct $name(f32);
    };
    (type $name:ident f64) => {
        #[derive(Debug)]
        pub struct $name(f64);
    };

    // String
    (type $name:ident String) => {
        #[derive(Debug)]
        pub struct $name(String);
    };

    // Bytes
    (type $name:ident Box<u8>) => {
        #[derive(Debug)]
        pub struct $name(Box<[u8]>);
    };

    // Copy
    (type $name:ident = $type:ty) => {
        pub type $name = $type;
    };

    //////////////////////////////////////////////////////////////////////////
    // Recursive Types
    //////////////////////////////////////////////////////////////////////////

    // Link
    (type $name:ident Link<$type:ty>) => {
        pub type $name = Link<$type>;
    };

    // List
    (type $name:ident [$elem_type:ty]) => {
        schema_typedef_list!($name $elem_type);
    };

    // Map
    (type $name:ident {$key:ty : $value:ty}) => {
        schema_typedef_map!($name { $key: $value });
    };

    // Struct
    (type $name:ident struct {}) => {
        schema_typedef_struct!($name {});
    };

    // Enum
    (type $name:ident enum {}) => {};

    // Union
    (type $name:ident union {}) => {};

    //////////////////////////////////////////////////////////////////////////
    // Advanced Types
    //////////////////////////////////////////////////////////////////////////

    //
    (advanced $name:ident) => {
        schema_typedef_advanced!($name)
    };
}

//////////////////////////////////////////////////////////////////////////
// Representations
//////////////////////////////////////////////////////////////////////////

#[macro_export(local_inner_macros)]
macro_rules! schema_repr {
    ($($schema:tt)*) => {};
}

#[cfg(test)]
mod tests {
    use crate::*;

    schema!(type Null null);
    schema!(type Bool bool);
    schema!(type Int8 i8);
    schema!(type Int16 i16);
    schema!(type Int32 i32);
    schema!(type Int64 i64);
    schema!(type Uint8 u8);
    schema!(type Uint16 u16);
    schema!(type Uint32 u32);
    schema!(type Uint64 u64);
    schema!(type Float32 f32);
    schema!(type Float64 f64);
    schema!(type TString String);
    schema!(type Bytes Box<u8>);
    schema!(type Bytes2 = Bytes);

    schema!(type Next Link<String>);
    schema!(type List [String]);
    schema!(type Map {String: u8});
    schema!(type A struct {});

    #[test]
    fn test_macro() {}
}
