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
mod primitive;
mod recursive;
mod typedef;

#[macro_export]
macro_rules! schema {
    ($($schema:tt)+) => {
        schema_typedef!($($schema)*);
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    // primitive types
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

    // recursive types
    schema!(type StringLink Link<String>);
    schema!(type List [TString]);
    schema!(type Map {String: List});

    // IPLD schema types and representations
    schema!(type MapMap {String: List} representation map);
    schema!(type MapStringpairs {String: List} representation stringpairs {
        innerDelim: ":",
        entryDelim: ","
    });
    schema!(type MapListpairs {String: List} representation listpairs);

    // struct
    schema!(type Struct struct {});

    // advanced representations

    #[test]
    fn test_macro() {}
}
