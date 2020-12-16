use crate::ast::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_encode(ast: &SchemaType) -> TokenStream {
    let (ident, body) = match ast {
        SchemaType::Struct(s) => (&s.name, gen_encode_struct(&s)),
        SchemaType::Union(u) => (&u.name, gen_encode_union(&u)),
    };

    quote! {
        impl libipld::codec::Encode<libipld::cbor::DagCborCodec> for #ident {
            fn encode<W: std::io::Write>(
                &self,
                c: libipld::cbor::DagCborCodec,
                w: &mut W,
            ) -> libipld::Result<()> {
                use libipld::codec::Encode;
                use libipld::cbor::encode::{write_null, write_u64};
                #body
            }
        }
    }
}

pub fn gen_decode(ast: &SchemaType) -> TokenStream {
    let ident = match ast {
        SchemaType::Struct(s) => &s.name,
        SchemaType::Union(u) => &u.name,
    };

    quote! {
        impl libipld::codec::Decode<libipld::cbor::DagCborCodec> for #ident {
            fn decode<R: std::io::Read>(
                c: libipld::cbor::DagCborCodec,
                r: &mut R,
            ) -> libipld::Result<Self> {
                libipld::cbor::decode::read(r)
            }
        }
    }
}

pub fn gen_try_read_cbor(ast: &SchemaType) -> TokenStream {
    let (ident, body) = match ast {
        SchemaType::Struct(s) => (&s.name, gen_try_read_cbor_struct(&s)),
        SchemaType::Union(u) => (&u.name, gen_try_read_cbor_union(&u)),
    };
    quote! {
        impl libipld::cbor::decode::TryReadCbor for #ident {
            fn try_read_cbor<R: std::io::Read>(
                r: &mut R,
                major: u8,
            ) -> libipld::Result<Option<Self>> {
                #body
            }
        }
    }
}

fn rename(name: &syn::Member, rename: Option<&String>) -> TokenStream {
    if let Some(rename) = rename {
        quote!(#rename)
    } else {
        let name = match name {
            syn::Member::Named(ident) => ident.to_string(),
            syn::Member::Unnamed(index) => index.index.to_string(),
        };
        quote!(#name)
    }
}

fn default(binding: &syn::Ident, default: Option<&syn::Expr>, tokens: TokenStream) -> TokenStream {
    if let Some(default) = default {
        quote! {
            if #binding != &#default {
                #tokens
            }
        }
    } else {
        tokens
    }
}

fn gen_encode_match(arms: impl Iterator<Item = TokenStream>) -> TokenStream {
    quote! {
        match *self {
            #(#arms,)*
        }
        Ok(())
    }
}

fn try_read_cbor(ty: TokenStream) -> TokenStream {
    quote! {{
        if let Some(t) = #ty::try_read_cbor(r, major)? {
            t
        } else {
            return Ok(None);
        }
    }}
}

fn gen_encode_struct(s: &Struct) -> TokenStream {
    let pat = &*s.pat;
    let body = gen_encode_struct_body(s);
    gen_encode_match(std::iter::once(quote!(#pat => { #body })))
}

fn gen_try_read_cbor_struct(_s: &Struct) -> TokenStream {
    quote!(Ok(None))
}

fn gen_encode_struct_body(s: &Struct) -> TokenStream {
    let len = s.fields.len() as u64;
    match s.repr {
        StructRepr::Map => {
            let fields = s.fields.iter().map(|field| {
                let key = rename(&field.name, field.rename.as_ref());
                let binding = &field.binding;
                default(
                    binding,
                    field.default.as_ref(),
                    quote! {
                        Encode::encode(#key, c, w)?;
                        Encode::encode(#binding, c, w)?;
                    },
                )
            });
            quote! {
                write_u64(w, 5, #len)?;
                #(#fields)*
            }
        }
        StructRepr::Tuple => {
            let fields = s.fields.iter().map(|field| {
                let binding = &field.binding;
                default(
                    binding,
                    field.default.as_ref(),
                    quote! {
                        Encode::encode(#binding, c, w)?;
                    },
                )
            });
            quote! {
                write_u64(w, 4, #len)?;
                #(#fields)*
            }
        }
        StructRepr::Value => {
            assert_eq!(s.fields.len(), 1);
            let field = &s.fields[0];
            let binding = &field.binding;
            default(
                binding,
                field.default.as_ref(),
                quote! {
                    Encode::encode(#binding, c, w)?;
                },
            )
        }
        StructRepr::Null => {
            assert_eq!(s.fields.len(), 0);
            quote!(write_null(w)?;)
        }
    }
}

fn gen_encode_union(u: &Union) -> TokenStream {
    let arms = u
        .variants
        .iter()
        .map(|s| {
            let pat = &*s.pat;
            let key = rename(&syn::Member::Named(s.name.clone()), s.rename.as_ref());
            let value = gen_encode_struct_body(s);
            match u.repr {
                UnionRepr::Keyed => {
                    quote! {
                        #pat => {
                            write_u64(w, 5, 1)?;
                            Encode::encode(#key, c, w)?;
                            #value
                        }
                    }
                }
                UnionRepr::Kinded => {
                    quote!(#pat => { #value })
                }
                UnionRepr::String => {
                    assert_eq!(s.repr, StructRepr::Null);
                    quote!(#pat => Encode::encode(#key, c, w)?)
                }
                UnionRepr::Int => {
                    assert_eq!(s.repr, StructRepr::Null);
                    quote!()
                }
            }
        })
        .collect::<Vec<_>>();
    if u.repr == UnionRepr::Int {
        quote!(Encode::encode(&(*self as u64), c, w))
    } else {
        gen_encode_match(arms.into_iter())
    }
}

fn gen_try_read_cbor_union(u: &Union) -> TokenStream {
    let (expr, expr_ref) = match u.repr {
        UnionRepr::Keyed => return quote!(Ok(None)),
        UnionRepr::Kinded => return quote!(Ok(None)),
        UnionRepr::String => (try_read_cbor(quote!(String)), quote!(key.as_str())),
        UnionRepr::Int => (try_read_cbor(quote!(u64)), quote!(key)),
    };
    let arms = u.variants.iter().map(|v| {
        let pat = &*v.pat;
        let value = match u.repr {
            UnionRepr::Keyed => unimplemented!(),
            UnionRepr::Kinded => unimplemented!(),
            UnionRepr::String => rename(&syn::Member::Named(v.name.clone()), v.rename.as_ref()),
            UnionRepr::Int => {
                quote!(x if x == #pat as u64)
            }
        };
        quote!(#value => #pat)
    });
    quote! {
        use libipld::error::{TypeError, TypeErrorType};
        let key = #expr;
        let key_ref = #expr_ref;
        let res = match key_ref {
            #(#arms,)*
            _ => return Err(TypeError::new(TypeErrorType::Key(key.to_string()), TypeErrorType::Null).into()),
        };
        Ok(Some(res))
    }
}
