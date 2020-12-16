use crate::ast::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_encode(ast: &SchemaType) -> TokenStream {
    let (ident, body) = match ast {
        SchemaType::Struct(s) => (&s.name, gen_encode_struct(&s)),
        SchemaType::Union(u) => (&u.name, gen_encode_union(&u)),
        SchemaType::Enum(e) => (&e.name, gen_encode_enum(&e)),
    };

    quote! {
        impl libipld::codec::Encode<libipld::cbor::DagCborCodec> for #ident {
            fn encode<W: std::io::Write>(
                &self,
                c: libipld::cbor::DagCborCodec,
                w: &mut W,
            ) -> libipld::Result<()> {
                use libipld::codec::Encode;
                use libipld::cbor::encode::write_u64;
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

fn gen_encode_struct(s: &Struct) -> TokenStream {
    let pat = &*s.pat;
    let body = gen_encode_struct_body(s);
    gen_encode_match(std::iter::once(quote!(#pat => { #body })))
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
    }
}

fn gen_encode_union(u: &Union) -> TokenStream {
    let name = &u.name;
    let arms = u.variants.iter().map(|s| {
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
        }
    });
    gen_encode_match(arms)
}

fn gen_encode_enum(e: &Enum) -> TokenStream {
    match e.repr {
        EnumRepr::String => {
            let arms = e.values.iter().map(|v| {
                let pat = &*v.pat;
                let value = rename(&syn::Member::Named(v.name.clone()), v.rename.as_ref());
                quote!(#pat => Encode::encode(#value, c, w)?)
            });
            gen_encode_match(arms)
        }
        EnumRepr::Int => {
            quote!(Encode::encode(&(*self as u64), c, w))
        }
    }
}

pub fn gen_decode(ast: &SchemaType) -> TokenStream {
    let ident = match ast {
        SchemaType::Struct(s) => &s.name,
        SchemaType::Union(u) => &u.name,
        SchemaType::Enum(e) => &e.name,
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
        SchemaType::Enum(e) => (&e.name, gen_try_read_cbor_enum(&e)),
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

fn try_read_cbor(ty: TokenStream) -> TokenStream {
    quote! {{
        if let Some(t) = #ty::try_read_cbor(r, major)? {
            t
        } else {
            return Ok(None);
        }
    }}
}

fn gen_try_read_cbor_struct(e: &Struct) -> TokenStream {
    quote!(Ok(None))
}

fn gen_try_read_cbor_union(e: &Union) -> TokenStream {
    quote!(Ok(None))
}

fn gen_try_read_cbor_enum(e: &Enum) -> TokenStream {
    let (expr, expr_ref) = match e.repr {
        EnumRepr::String => (try_read_cbor(quote!(String)), quote!(key.as_str())),
        EnumRepr::Int => (try_read_cbor(quote!(u64)), quote!(key)),
    };
    let arms = e.values.iter().map(|v| {
        let pat = &*v.pat;
        let value = match e.repr {
            EnumRepr::String => rename(&syn::Member::Named(v.name.clone()), v.rename.as_ref()),
            EnumRepr::Int => {
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
