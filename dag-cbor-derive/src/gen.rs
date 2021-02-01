use crate::ast::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_encode(ast: &SchemaType) -> TokenStream {
    let (ident, generics, body) = match ast {
        SchemaType::Struct(s) => (&s.name, s.generics.as_ref().unwrap(), gen_encode_struct(&s)),
        SchemaType::Union(u) => (&u.name, &u.generics, gen_encode_union(&u)),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let trait_name = quote!(libipld::codec::Encode<libipld::cbor::DagCborCodec>);

    quote! {
        impl#impl_generics #trait_name for #ident #ty_generics #where_clause {
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
    let (ident, generics) = match ast {
        SchemaType::Struct(s) => (&s.name, s.generics.as_ref().unwrap()),
        SchemaType::Union(u) => (&u.name, &u.generics),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let trait_name = quote!(libipld::codec::Decode<libipld::cbor::DagCborCodec>);

    quote! {
        impl#impl_generics #trait_name for #ident #ty_generics #where_clause {
            fn decode<R: std::io::Read + std::io::Seek>(
                c: libipld::cbor::DagCborCodec,
                r: &mut R,
            ) -> libipld::Result<Self> {
                libipld::cbor::decode::read(r)
            }
        }
    }
}

pub fn gen_try_read_cbor(ast: &SchemaType) -> TokenStream {
    let (ident, generics, body) = match ast {
        SchemaType::Struct(s) => (
            &s.name,
            s.generics.as_ref().unwrap(),
            gen_try_read_cbor_struct(&s),
        ),
        SchemaType::Union(u) => (&u.name, &u.generics, gen_try_read_cbor_union(&u)),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let trait_name = quote!(libipld::cbor::decode::TryReadCbor);

    quote! {
        impl#impl_generics #trait_name for #ident #ty_generics #where_clause {
            fn try_read_cbor<R: std::io::Read + std::io::Seek>(
                r: &mut R,
                major: u8,
            ) -> libipld::Result<Option<Self>> {
                use libipld::cbor::decode::{read_len, read_u8, TryReadCbor};
                use libipld::cbor::error::{LengthOutOfRange, MissingKey, UnexpectedCode, UnexpectedKey};
                use libipld::codec::Decode;
                use libipld::error::Result;
                let c = libipld::cbor::DagCborCodec;
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

fn gen_encode_struct_body(s: &Struct) -> TokenStream {
    match s.repr {
        StructRepr::Map => {
            let len = s.fields.len() as u64;
            let dfields = s.fields.iter().filter_map(|field| {
                if let Some(default) = field.default.as_ref() {
                    let binding = &field.binding;
                    let default = &*default;
                    Some(quote! {
                        if #binding == &#default {
                            len -= 1;
                        }
                    })
                } else {
                    None
                }
            });
            let fields = s.fields.iter().map(|field| {
                let key = rename(&field.name, field.rename.as_ref());
                let binding = &field.binding;
                default(
                    binding,
                    field.default.as_deref(),
                    quote! {
                        Encode::encode(#key, c, w)?;
                        Encode::encode(#binding, c, w)?;
                    },
                )
            });
            quote! {
                let mut len = #len;
                #(#dfields)*
                write_u64(w, 5, len)?;
                #(#fields)*
            }
        }
        StructRepr::Tuple => {
            let len = s.fields.len() as u64;
            let fields = s.fields.iter().map(|field| {
                let binding = &field.binding;
                quote! {
                    Encode::encode(#binding, c, w)?;
                }
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
                field.default.as_deref(),
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

#[allow(clippy::needless_collect)]
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

fn gen_try_read_cbor_struct(s: &Struct) -> TokenStream {
    let len = s.fields.len();
    let construct = &*s.construct;
    match s.repr {
        StructRepr::Map => {
            let binding: Vec<_> = s.fields.iter().map(|field| &field.binding).collect();
            let key: Vec<_> = s
                .fields
                .iter()
                .map(|field| rename(&field.name, field.rename.as_ref()))
                .collect();
            let fields: Vec<_> = s
                .fields
                .iter()
                .map(|field| {
                    let binding = &field.binding;
                    let key = rename(&field.name, field.rename.as_ref());
                    if let Some(default) = field.default.as_ref() {
                        quote!(let #binding = #binding.unwrap_or(#default);)
                    } else {
                        quote!(let #binding = #binding.ok_or(MissingKey::new::<Self>(#key))?;)
                    }
                })
                .collect();
            quote! {
                match major {
                    0xa0..=0xbb => {
                        let len = read_len(r, major - 0xa0)?;
                        if len > #len {
                            return Err(LengthOutOfRange::new::<Self>().into());
                        }
                        #(let mut #binding = None;)*
                        for _ in 0..len {
                            let mut key: String = Decode::decode(c, r)?;
                            match key.as_str() {
                                #(#key => { #binding = Some(Decode::decode(c, r)?); })*
                                _ => {
                                    libipld::Ipld::decode(c, r)?;
                                    //return Err(UnexpectedKey::new::<Self>(key).into()),
                                }
                            }
                        }

                        #(#fields)*

                        return Ok(Some(#construct));
                    }
                    0xbf => {
                        #(let mut #binding = None;)*
                        loop {
                            let major = read_u8(r)?;
                            if major == 0xff {
                                break;
                            }
                            if let Some(key) = String::try_read_cbor(r, major)? {
                                match key.as_str() {
                                    #(#key => { #binding = Some(Decode::decode(c, r)?); })*
                                    _ => {
                                        libipld::Ipld::decode(c, r)?;
                                        //return Err(UnexpectedKey::new::<Self>(key).into()),
                                    }
                                }
                            } else {
                                return Err(UnexpectedCode::new::<Self>(major).into());
                            }
                        }

                        #(#fields)*

                        return Ok(Some(#construct));
                    }
                    _ => Ok(None),
                }
            }
        }
        StructRepr::Tuple => {
            let fields = s.fields.iter().map(|field| {
                let binding = &field.binding;
                quote! {
                    let #binding = Decode::decode(c, r)?;
                }
            });
            quote! {
                match major {
                    0x80..=0x9b => {
                        let len = read_len(r, major - 0x80)?;
                        if len != #len {
                            return Err(LengthOutOfRange::new::<Self>().into());
                        }
                        #(#fields)*
                        return Ok(Some(#construct));
                    }
                    _ => Ok(None),
                }
            }
        }
        StructRepr::Value => {
            assert_eq!(s.fields.len(), 1);
            let binding = &s.fields[0].binding;
            quote! {
                if let Some(#binding) = TryReadCbor::try_read_cbor(r, major)? {
                    return Ok(Some(#construct));
                } else {
                    Ok(None)
                }
            }
        }
        StructRepr::Null => {
            assert_eq!(s.fields.len(), 0);
            quote! {
                match major {
                    0xf6..=0xf7 => {
                        return Ok(Some(#construct));
                    }
                    _ => Ok(None),
                }
            }
        }
    }
}

fn gen_try_read_cbor_union(u: &Union) -> TokenStream {
    match u.repr {
        UnionRepr::Keyed => {
            let variants = u.variants.iter().map(|s| {
                let key = rename(&syn::Member::Named(s.name.clone()), s.rename.as_ref());
                let parse = gen_try_read_cbor_struct(s);
                quote! {
                    if key.as_str() == #key {
                        let major = read_u8(r)?;
                        let res: Result<Option<Self>> = #parse;
                        res?;
                    }
                }
            });
            quote! {
                if major != 0xa1 {
                    return Ok(None);
                }
                let key: String = Decode::decode(c, r)?;
                #(#variants;)*
                Err(UnexpectedKey::new::<Self>(key).into())
            }
        }
        UnionRepr::Kinded => {
            let variants = u.variants.iter().map(|s| {
                let parse = gen_try_read_cbor_struct(s);
                quote! {
                    let res: Result<Option<Self>> = #parse;
                    res?;
                }
            });
            quote! {
                #(#variants;)*
                Err(UnexpectedCode::new::<Self>(major).into())
            }
        }
        UnionRepr::String => {
            let arms = u.variants.iter().map(|v| {
                let pat = &*v.pat;
                let value = rename(&syn::Member::Named(v.name.clone()), v.rename.as_ref());
                quote!(#value => #pat)
            });
            let parse = try_read_cbor(quote!(String));
            quote! {
                let key = #parse;
                let res = match key.as_str() {
                    #(#arms,)*
                    _ => return Err(UnexpectedKey::new::<Self>(key).into()),
                };
                Ok(Some(res))
            }
        }
        UnionRepr::Int => {
            let arms = u.variants.iter().map(|v| {
                let pat = &*v.pat;
                quote!(x if x == #pat as u64 => #pat)
            });
            let parse = try_read_cbor(quote!(u64));
            quote! {
                let key = #parse;
                let res = match key {
                    #(#arms,)*
                    _ => return Err(UnexpectedKey::new::<Self>(key.to_string()).into()),
                };
                Ok(Some(res))
            }
        }
    }
}
