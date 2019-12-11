//! `schema!` macro.
//! Define a native type modelling an IPLD Schema and it's Representation.
//!
//! ```edition2018
//! # use libipld_schema;
//! ```
//!
//! TODO: next steps:
//! - support pub/pub(crate) and additional #[derive(...)] statements
//! - anything can have an advanced representation, so add support to all types
pub use crate::link::Link;
pub use async_trait::async_trait;
pub use libipld::*;
pub use std::collections::BTreeMap;

mod advanced;
mod link;
mod primitive;
mod recursive;

#[macro_export(local_inner_macros)]
macro_rules! schema {
    ($($schema:tt)+) => {
        schema_typedef!($($schema)*);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! schema_typedef {
    //////////////////////////////////////////////////////////////////////////
    // Primitive Types
    //////////////////////////////////////////////////////////////////////////

    // Null
    (type $name:ident null) => {
        schema_typedef_null!($name);
    };

    // Bool
    (type $name:ident bool) => {
        schema_typedef_bool!($name);
    };

    // Integer
    (type $name:ident i8) => {
        schema_typedef_num!($name i8);
    };
    (type $name:ident i16) => {
        schema_typedef_num!($name i16);
    };
    (type $name:ident i32) => {
        schema_typedef_num!($name i32);
    };
    (type $name:ident i64) => {
        schema_typedef_num!($name i64);
    };
    (type $name:ident u8) => {
        schema_typedef_num!($name u8);
    };
    (type $name:ident u16) => {
        schema_typedef_num!($name u16);
    };
    (type $name:ident u32) => {
        schema_typedef_num!($name u32);
    };
    (type $name:ident u64) => {
        schema_typedef_num!($name u64);
    };

    // Float
    (type $name:ident f32) => {
        schema_typedef_num!($name f32);
    };
    (type $name:ident f64) => {
        schema_typedef_num!($name f64);
    };

    // String
    (type $name:ident String) => {
        schema_typedef_str!($name);
    };

    // Bytes
    (type $name:ident bytes) => {
        schema_typedef_bytes!($name);
    };

    // Copy
    (type $name:ident = $type:ty) => {
        type $name = $type;
    };

    //////////////////////////////////////////////////////////////////////////
    // Recursive Types
    //////////////////////////////////////////////////////////////////////////

    // Link
    (type $name:ident Link<$type:ty>) => {
        schema_typedef_link!($name $type);
    };

    // List
    (type $name:ident [ $elem_type:ty ]) => {
        schema_typedef_list!($name $elem_type);
    };

    // Map
    (type $name:ident { $key:ty : $value:ty }) => {
        schema_typedef_map!($name { $key: $value });
    };
    (type $name:ident { $key:ty : $value:ty } representation map) => {
        schema_typedef_map!($name { $key: $value });
    };
    (type $name:ident { $key:ty : $value:ty } representation stringpairs {
        innerDelim : $inner:expr,
        entryDelim : $entry:expr
    }) => {
        schema_typedef_map!($name { $key: $value } { $inner, $entry });
    };
    (type $name:ident { $key:ty : $value:ty } representation listpairs) => {
        schema_typedef_map!($name { $key: $value } @listpairs);
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
    schema!(type Bytes1 bytes);
    schema!(type Bytes2 = Bytes1);

    schema!(type Next Link<String>);
    schema!(type List [String]);
    schema!(type Map1 {String: u8} representation map);
    schema!(type Map2 {String: u8} representation stringpairs {
        innerDelim: ":",
        entryDelim: ","
    });
    schema!(type Map3 {String: u8} representation listpairs);
    schema!(type A struct {});

    #[test]
    fn test_macro() {}
}
