#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_list {
    ($name:ident $elem_type:ty) => {
        #[derive(DagCbor)]
        pub struct $name(Vec<$elem_type>);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_map {
    ($name:ident {$key:ty : $value:ty}) => {
        #[derive(DagCbor)]
        pub struct $name(std::collections::BTreeMap<$key, $value>);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! schema_typedef_struct {
    ($name:ident {}) => {
        #[derive(DagCbor)]
        pub struct $name;
    };
}
