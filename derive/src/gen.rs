use crate::ast::{SchemaType, Struct, StructRepr, Union, UnionRepr};
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_deserialize(ast: &SchemaType) -> TokenStream {
    match ast {
        SchemaType::Union(union) => {
            let ident = &union.name;
            let body = gen_deserialize_union(union);

            quote! {
                const _: () = {
                    extern crate serde as _serde;
                    extern crate libipld_core as _libipld_core;
                    impl<'de> _serde::de::Deserialize<'de> for #ident {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: _serde::de::Deserializer<'de>,
                        {
                            #body
                        }
                    }
                };
            }
        }
        SchemaType::Struct(_s) => unimplemented!(),
    }
}

fn gen_deserialize_struct(s: &Struct) -> TokenStream {
    let construct = &*s.construct;
    match s.repr {
        StructRepr::Map => {
            let struct_name = &s.name;
            let fields_with_type: Vec<_> = s
                .fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let ty = &field.ty;
                    quote! { #name: #ty }
                })
                .collect();
            let bindings: Vec<_> = s
                .fields
                .iter()
                .map(|field| {
                    let name = &field.name;
                    let binding = &field.binding;
                    quote! { #name: #binding }
                })
                .collect();
            // Create a temporary struct that matches the struct variant of the enum and use that
            // to deserialize it into the correct shape.
            quote! {
                #[derive(_serde::Deserialize)]
                struct #struct_name {
                    #(#fields_with_type,)*
                }
                if let Ok(#struct_name { #(#bindings,)* }) = _serde::de::Deserialize::deserialize(ipld.clone()) {
                    return Ok(#construct);
                }
            }
        }
        StructRepr::Tuple => {
            let fields = s.fields.iter().map(|field| {
                let binding = &field.binding;
                quote! { #binding }
            });
            // Wrap it in a tuple if there is more than one field
            let binding = if s.fields.len() == 1 {
                quote! { #(#fields,)* }
            } else {
                quote! { (#(#fields,)*) }
            };
            quote! {
                if let Ok(#binding) = _serde::de::Deserialize::deserialize(ipld.clone()) {
                    return Ok(#construct);
                }
            }
        }
        StructRepr::Value => {
            assert_eq!(s.fields.len(), 1);
            let field = &s.fields[0].binding;
            quote! {
                if let Ok(#field) = _serde::de::Deserialize::deserialize(ipld.clone()) {
                    return Ok(#construct);
                }
            }
        }
        StructRepr::Null => {
            assert_eq!(s.fields.len(), 0);
            quote! {
                if let Ok(()) = _serde::de::Deserialize::deserialize(ipld.clone()) {
                    return Ok(#construct);
                }
            }
        }
    }
}

fn gen_deserialize_union(u: &Union) -> TokenStream {
    match u.repr {
        UnionRepr::Keyed => {
            unimplemented!();
        }
        UnionRepr::Kinded => {
            let variants = u.variants.iter().map(|s| {
                let parse = gen_deserialize_struct(s);
                quote! { #parse }
            });
            quote! {
               let deserialized_result = _libipld_core::ipld::Ipld::deserialize(deserializer);
               if let Ok(ipld) = deserialized_result {
                   #(#variants;)*
               }
               Err(_serde::de::Error::custom("No matching enum variant found"))
            }
        }
        UnionRepr::String => {
            unimplemented!();
        }
        UnionRepr::Int => {
            unimplemented!();
        }
        UnionRepr::IntTuple => {
            unimplemented!();
        }
    }
}
