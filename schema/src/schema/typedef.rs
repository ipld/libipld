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
    // TODO: ? create a new struct that wraps the copied and delegates?
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
}
