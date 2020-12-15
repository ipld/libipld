use crate::ast::*;
use crate::attr::{Attrs, DeriveAttr, FieldAttr};
use syn::parse::Parse;
use syn::spanned::Spanned;
use synstructure::Structure;

pub fn parse(s: &Structure) -> SchemaType {
    match &s.ast().data {
        syn::Data::Struct(d) => {
            let repr = parse_struct_repr(&s.ast().attrs);
            SchemaType::Struct(parse_struct(&s.ast().ident, &d.fields, repr))
        }
        syn::Data::Enum(d) => match parse_rust_enum_repr(&s.ast().attrs) {
            RustEnumRepr::Union(repr) => SchemaType::Union(parse_union(&s.ast().ident, d, repr)),
            RustEnumRepr::Enum(repr) => SchemaType::Enum(parse_enum(&s.ast().ident, d, repr)),
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

fn parse_struct(ident: &syn::Ident, fields: &syn::Fields, repr: Option<StructRepr>) -> Struct {
    Struct {
        name: ident.clone(),
        fields: fields
            .iter()
            .enumerate()
            .map(|(i, field)| parse_field(i, field))
            .collect(),
        repr: repr.unwrap_or_else(|| {
            if let syn::Fields::Named(_) = &fields {
                StructRepr::Map
            } else {
                StructRepr::Tuple
            }
        }),
    }
}

fn parse_union(ident: &syn::Ident, s: &syn::DataEnum, repr: UnionRepr) -> Union {
    Union {
        name: ident.clone(),
        variants: s
            .variants
            .iter()
            .map(|v| {
                let repr = parse_struct_repr(&v.attrs);
                parse_struct(&v.ident, &v.fields, repr)
            })
            .collect(),
        repr,
    }
}

fn parse_enum(ident: &syn::Ident, s: &syn::DataEnum, repr: EnumRepr) -> Enum {
    Enum {
        name: ident.clone(),
        values: s
            .variants
            .iter()
            .map(|v| {
                assert_eq!(v.fields, syn::Fields::Unit);
                let mut value = EnumValue {
                    name: v.ident.clone(),
                    rename: None,
                };
                for attr in parse_attrs::<FieldAttr>(&v.attrs) {
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

fn parse_field(i: usize, ast: &syn::Field) -> StructField {
    let mut field = StructField {
        name: match ast.ident.as_ref() {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index {
                index: i as _,
                span: ast.ty.span(),
            }),
        },
        rename: None,
        nullable: false,
        optional: false,
        implicit: None,
        default: None,
    };
    for attr in parse_attrs::<FieldAttr>(&ast.attrs) {
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
    use proc_macro2::{Span, TokenStream};
    use quote::{format_ident, quote};

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
                }],
                repr: StructRepr::Map,
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
                    name: syn::Member::Unnamed(syn::Index {
                        index: 0,
                        span: Span::call_site()
                    }),
                    rename: None,
                    default: None,
                    nullable: false,
                    optional: false,
                    implicit: None,
                }],
                repr: StructRepr::Tuple,
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
                    },
                    Struct {
                        name: format_ident!("Tuple"),
                        fields: vec![0.into()],
                        repr: StructRepr::Tuple,
                    },
                    Struct {
                        name: format_ident!("Struct"),
                        fields: vec![format_ident!("value").into()],
                        repr: StructRepr::Map,
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
                    rename: Some("test".into())
                }],
                repr: EnumRepr::String,
            })
        );
    }
}
