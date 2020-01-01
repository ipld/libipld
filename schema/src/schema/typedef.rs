#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! typedef {
    //////////////////////////////////////////////////////////////////////////
    // IPLD Primitive Data Types
    //////////////////////////////////////////////////////////////////////////

    // Null
    (type $name:ident null) => {
        typedef_null!($name);
    };

    // Bool
    (type $name:ident bool) => {
        typedef_bool!($name);
    };

    // Integer
    (type $name:ident int) => {
        typedef_num!($name i32);
    };
    (type $name:ident i8) => {
        typedef_num!($name i8);
    };
    (type $name:ident i16) => {
        typedef_num!($name i16);
    };
    (type $name:ident i32) => {
        typedef_num!($name i32);
    };
    (type $name:ident i64) => {
        typedef_num!($name i64);
    };
    (type $name:ident u8) => {
        typedef_num!($name u8);
    };
    (type $name:ident u16) => {
        typedef_num!($name u16);
    };
    (type $name:ident u32) => {
        typedef_num!($name u32);
    };
    (type $name:ident u64) => {
        typedef_num!($name u64);
    };

    // Float
    (type $name:ident float) => {
        typedef_num!($name f64);
    };
    (type $name:ident f32) => {
        typedef_num!($name f32);
    };
    (type $name:ident f64) => {
        typedef_num!($name f64);
    };

    // String
    (type $name:ident String) => {
        typedef_str!($name);
    };

    // Bytes
    (type $name:ident bytes) => {
        typedef_bytes!($name);
    };

    // Copy
    // TODO: ? create a new struct that wraps the copied and delegates?
    (type $name:ident = $type:ty) => {
        type $name = $type;
    };

    //////////////////////////////////////////////////////////////////////////
    // IPLD Recursive Data Types
    //////////////////////////////////////////////////////////////////////////

    // Link
    (type $name:ident Link<$type:ty>) => {
        typedef_link!($name $type);
    };

    // List
    (type $name:ident [ $elem_type:ty ]) => {
        typedef_list!($name $elem_type);
    };

    // Map
    (type $name:ident { $key:ty : $value:ty }) => {
        typedef_map!($name { $key: $value });
    };
    // basic map representation
    (type $name:ident { $key:ty : $value:ty } representation map) => {
        typedef_map!($name { $key: $value });
    };
    // TODO: stringpairs map representation
    (type $name:ident { $key:ty : $value:ty } representation stringpairs {
        innerDelim : $inner:expr,
        entryDelim : $entry:expr
    }) => {
        typedef_map!($name { $key: $value } @stringpairs $inner, $entry);
    };
    // TODO: listpairs map representation
    (type $name:ident { $key:ty : $value:ty } representation listpairs) => {
        typedef_map!($name { $key: $value } @listpairs);
    };

    //////////////////////////////////////////////////////////////////////////
    // IPLD Schema Types
    //////////////////////////////////////////////////////////////////////////

    // TODO: Struct
    (type $name:ident struct {/* fields */}) => {
        typedef_struct!($name {/* fields */});
    };
    // TODO: basic map representation
    (type $name:ident struct {/* fields */} representation map) => {
        typedef_struct!($name {/* fields */});
    };
    // TODO: struct tuple representation
    (type $name:ident struct {/* fields */} representation tuple) => {
        typedef_struct!($name {/* fields */} @tuple);
    };
    // TODO: struct stringpairs representation
    (type $name:ident struct {/* fields */} representation stringpairs {
        innerDelim : $inner:expr,
        entryDelim : $entry:expr
    }) => {
        typedef_struct!($name {/* fields */} @stringpairs $inner, $entry);
    };
    // TODO: struct stringjoin representation
    (type $name:ident struct {/* fields */} representation stringjoin {
        join : $joiner:expr
    }) => {
        typedef_struct!($name {/* fields */} @stringjoin $joiner);
    };
    // TODO: struct listpairs representation
    (type $name:ident struct {/* fields */} representation listpairs) => {
        typedef_struct!($name {/* fields */} @listpairs);
    };

    // TODO: Enum
    (type $name:ident enum {/* fields */}) => {
        typedef_enum!($name {/* fields */});
    };
    // TODO: basic enum representation
    (type $name:ident enum {/* fields */} representation string) => {
        typedef_enum!($name {/* fields */});
    };
    // TODO: enum int representation
    (type $name:ident enum {/* fields */} representation int) => {
        typedef_enum!($name {/* fields */} @int);
    };

    // TODO: Union
    (type $name:ident union {/* fields */}) => {
        typedef_union!($name {/* fields */});
    };
    // TODO: union keyed representation
    (type $name:ident union {/* fields */} representation keyed) => {
        typedef_union!($name {/* fields */} @keyed);
    };
    // TODO: union kinded representation
    (type $name:ident union {/* fields */} representation kinded) => {
        typedef_union!($name {/* fields */} @kinded);
    };
    // TODO: union envelope representation
    (type $name:ident union {/* fields */} representation envelope {
        discriminantKey : $discriminant:expr,
        contentKey      : $content:expr
    }) => {
        typedef_union!($name {/* fields */} @envelope $discriminant, $content);
    };
    // TODO: union inline representation
    (type $name:ident union {/* fields */} representation inline {
        discriminantKey : $discriminant:expr
    }) => {
        typedef_union!($name {/* fields */} @inline $discriminant);
    };
    // TODO: union byteprefix representation
    (type $name:ident union {/* fields */} representation byteprefix) => {
        typedef_union!($name {/* fields */} @byteprefix);
    };
}
