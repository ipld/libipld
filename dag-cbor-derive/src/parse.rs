use crate::ast::*;
use crate::attr::{Attrs, DeriveAttr, FieldAttr};
use syn::parse::Parse;
use syn::spanned::Spanned;
use synstructure::{BindingInfo, Structure, VariantInfo};

pub fn parse(s: &Structure) -> SchemaType {
    match &s.ast().data {
        syn::Data::Struct(_) => SchemaType::Struct(parse_struct(&s.variants()[0])),
        syn::Data::Enum(_) => match parse_rust_enum_repr(&s.ast().attrs) {
            RustEnumRepr::Union(repr) => {
                SchemaType::Union(parse_union(&s.ast().ident, s.variants(), repr))
            }
            RustEnumRepr::Enum(repr) => {
                SchemaType::Enum(parse_enum(&s.ast().ident, s.variants(), repr))
            }
        },
        syn::Data::Union(_) => unimplemented!(),
    }
}

fn parse_attrs<T: Parse>(ast: &[syn::Attribute]) -> Vec<T> {
    let mut derive_attrs = Vec::with_capacity(ast.len());
    for attr in ast {
        let attrs: Result<Attrs<T>, _> = syn::parse2(attr.tokens.clone());
        if let Ok(attrs) = attrs {
            for attr in attrs.attrs {
                derive_attrs.push(attr);
            }
        }
    }
    derive_attrs
}

fn parse_struct_repr(ast: &[syn::Attribute]) -> Option<StructRepr> {
    let attrs = parse_attrs::<DeriveAttr>(ast);
    let mut repr = None;
    for DeriveAttr::Repr(attr) in attrs {
        repr = Some(match attr.value.value().as_str() {
            "map" => StructRepr::Map,
            "tuple" => StructRepr::Tuple,
            _ => unimplemented!(),
        })
    }
    repr
}

enum RustEnumRepr {
    Union(UnionRepr),
    Enum(EnumRepr),
}

fn parse_rust_enum_repr(ast: &[syn::Attribute]) -> RustEnumRepr {
    let attrs = parse_attrs::<DeriveAttr>(ast);
    let mut repr = None;
    for DeriveAttr::Repr(attr) in attrs {
        repr = Some(match attr.value.value().as_str() {
            "keyed" => RustEnumRepr::Union(UnionRepr::Keyed),
            "kinded" => RustEnumRepr::Union(UnionRepr::Kinded),
            "string" => RustEnumRepr::Enum(EnumRepr::String),
            "int" => RustEnumRepr::Enum(EnumRepr::Int),
            _ => unimplemented!(),
        })
    }
    repr.unwrap_or(RustEnumRepr::Union(UnionRepr::Keyed))
}

fn parse_struct(v: &VariantInfo) -> Struct {
    let repr = parse_struct_repr(&v.ast().attrs);
    Struct {
        name: v.ast().ident.clone(),
        fields: v
            .bindings()
            .iter()
            .enumerate()
            .map(|(i, binding)| parse_field(i, binding))
            .collect(),
        repr: repr.unwrap_or_else(|| {
            if let syn::Fields::Named(_) = &v.ast().fields {
                StructRepr::Map
            } else {
                StructRepr::Tuple
            }
        }),
        pat: TokenStreamEq(v.pat()),
    }
}

fn parse_union(ident: &syn::Ident, v: &[VariantInfo], repr: UnionRepr) -> Union {
    Union {
        name: ident.clone(),
        variants: v.iter().map(|v| parse_struct(v)).collect(),
        repr,
    }
}

fn parse_enum(ident: &syn::Ident, v: &[VariantInfo], repr: EnumRepr) -> Enum {
    Enum {
        name: ident.clone(),
        values: v
            .iter()
            .map(|v| {
                assert_eq!(*v.ast().fields, syn::Fields::Unit);
                let mut value = EnumValue {
                    name: v.ast().ident.clone(),
                    rename: None,
                    pat: TokenStreamEq(v.pat()),
                };
                for attr in parse_attrs::<FieldAttr>(&v.ast().attrs) {
                    match attr {
                        FieldAttr::Rename(attr) => value.rename = Some(attr.value.value()),
                        _ => unimplemented!(),
                    }
                }
                value
            })
            .collect(),
        repr,
    }
}

fn parse_field(i: usize, b: &BindingInfo) -> StructField {
    let mut field = StructField {
        name: match b.ast().ident.as_ref() {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index {
                index: i as _,
                span: b.ast().ty.span(),
            }),
        },
        rename: None,
        nullable: false,
        optional: false,
        implicit: None,
        default: None,
        binding: b.binding.clone(),
    };
    for attr in parse_attrs::<FieldAttr>(&b.ast().attrs) {
        match attr {
            FieldAttr::Rename(attr) => field.rename = Some(attr.value.value()),
            FieldAttr::Nullable(_) => field.nullable = true,
            FieldAttr::Optional(_) => field.optional = true,
            FieldAttr::Implicit(attr) => field.implicit = Some(attr.value),
            FieldAttr::Default(attr) => field.default = Some(attr.value),
        }
    }
    field
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    macro_rules! format_index {
        ($i:expr) => {
            syn::Index {
                index: $i as _,
                span: proc_macro2::Span::call_site(),
            }
        };
    }

    fn ast(ts: TokenStream) -> SchemaType {
        let d = syn::parse2(ts).unwrap();
        let s = Structure::new(&d);
        parse(&s)
    }

    #[test]
    fn test_struct_repr_map() {
        let ast = ast(quote! {
            #[derive(DagCbor)]
            #[ipld(repr = "map")]
            struct Map {
                #[ipld(rename = "other", default = false)]
                field: bool,
            }
        });

        assert_eq!(
            ast,
            SchemaType::Struct(Struct {
                name: format_ident!("Map"),
                fields: vec![StructField {
                    name: syn::Member::Named(format_ident!("field")),
                    rename: Some("other".to_string()),
                    default: Some(syn::parse2(quote!(false)).unwrap()),
                    nullable: false,
                    optional: false,
                    implicit: None,
                    binding: format_ident!("__binding_0"),
                }],
                repr: StructRepr::Map,
                pat: TokenStreamEq(quote!(Map { field: ref __binding_0, })),
            })
        );
    }

    #[test]
    fn test_struct_repr_tuple() {
        let ast = ast(quote! {
            #[derive(DagCbor)]
            #[ipld(repr = "tuple")]
            struct Tuple(bool);
        });

        assert_eq!(
            ast,
            SchemaType::Struct(Struct {
                name: format_ident!("Tuple"),
                fields: vec![StructField {
                    name: syn::Member::Unnamed(format_index!(0)),
                    rename: None,
                    default: None,
                    nullable: false,
                    optional: false,
                    implicit: None,
                    binding: format_ident!("__binding_0"),
                }],
                repr: StructRepr::Tuple,
                pat: TokenStreamEq(quote!(Tuple(ref __binding_0,))),
            })
        );
    }

    #[test]
    fn test_struct_repr_default() {
        let ast = ast(quote! {
            #[derive(DagCbor)]
            struct Map;
        });

        assert_eq!(
            ast,
            SchemaType::Struct(Struct {
                name: format_ident!("Map"),
                fields: Default::default(),
                repr: StructRepr::Tuple,
                pat: TokenStreamEq(quote!(Map)),
            })
        );
    }

    #[test]
    fn test_union_repr_default() {
        let ast = ast(quote! {
            #[derive(DagCbor)]
            enum Union {
                Unit,
                Tuple(bool),
                Struct { value: bool },
            }
        });

        assert_eq!(
            ast,
            SchemaType::Union(Union {
                name: format_ident!("Union"),
                variants: vec![
                    Struct {
                        name: format_ident!("Unit"),
                        fields: vec![],
                        repr: StructRepr::Tuple,
                        pat: TokenStreamEq(quote!(Union::Unit)),
                    },
                    Struct {
                        name: format_ident!("Tuple"),
                        fields: vec![StructField {
                            name: syn::Member::Unnamed(format_index!(0)),
                            rename: None,
                            default: None,
                            nullable: false,
                            optional: false,
                            implicit: None,
                            binding: format_ident!("__binding_0"),
                        }],
                        repr: StructRepr::Tuple,
                        pat: TokenStreamEq(quote!(Union::Tuple(ref __binding_0,))),
                    },
                    Struct {
                        name: format_ident!("Struct"),
                        fields: vec![StructField {
                            name: syn::Member::Named(format_ident!("value")),
                            rename: None,
                            default: None,
                            nullable: false,
                            optional: false,
                            implicit: None,
                            binding: format_ident!("__binding_0"),
                        }],
                        repr: StructRepr::Map,
                        pat: TokenStreamEq(quote!(Union::Struct { value: ref __binding_0, })),
                    }
                ],
                repr: UnionRepr::Keyed,
            })
        );
    }

    #[test]
    fn test_enum_repr_string() {
        let ast = ast(quote! {
            #[derive(DagCbor)]
            #[ipld(repr = "string")]
            enum Enum {
                #[ipld(rename = "test")]
                Variant,
            }
        });

        assert_eq!(
            ast,
            SchemaType::Enum(Enum {
                name: format_ident!("Enum"),
                values: vec![EnumValue {
                    name: format_ident!("Variant"),
                    rename: Some("test".into()),
                    pat: TokenStreamEq(quote!(Enum::Variant)),
                }],
                repr: EnumRepr::String,
            })
        );
    }
}
