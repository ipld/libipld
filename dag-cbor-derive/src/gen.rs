use crate::ast::*;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_encode(ast: &SchemaType) -> TokenStream {
    let (ident, body) = match ast {
        SchemaType::Struct(_) => unimplemented!(),
        SchemaType::Union(_) => unimplemented!(),
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
                #body
            }
        }
    }
}

fn gen_encode_enum(e: &Enum) -> TokenStream {
    match e.repr {
        EnumRepr::String => {
            let arms = e.values.iter().map(|v| {
                let pat = &*v.pat;
                let value = if let Some(rename) = v.rename.as_ref() {
                    quote!(#rename)
                } else {
                    let name = v.name.to_string();
                    quote!(#name)
                };
                quote!(#pat => Encode::encode(#value, c, w)?)
            });
            quote! {
                match *self {
                    #(#arms,)*
                };
                Ok(())
            }
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
        SchemaType::Struct(_) => unimplemented!(),
        SchemaType::Union(_) => unimplemented!(),
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

fn gen_try_read_cbor_enum(e: &Enum) -> TokenStream {
    let (expr, expr_ref) = match e.repr {
        EnumRepr::String => (try_read_cbor(quote!(String)), quote!(key.as_str())),
        EnumRepr::Int => (try_read_cbor(quote!(u64)), quote!(key)),
    };
    let arms = e.values.iter().map(|v| {
        let pat = &*v.pat;
        let value = match e.repr {
            EnumRepr::String => {
                if let Some(rename) = v.rename.as_ref() {
                    quote!(#rename)
                } else {
                    let name = v.name.to_string();
                    quote!(#name)
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::tests::ast;

    #[test]
    fn encode_string_enum() {
        let e = gen_encode(&ast(quote! {
            #[derive(DagCbor)]
            #[ipld(repr = "string")]
            enum Enum {
                #[ipld(rename = "test")]
                Variant,
                Other,
            }
        }))
        .to_string();
        assert_eq!(
            e,
            quote! {
                impl libipld::codec::Encode<libipld::cbor::DagCborCodec> for Enum {
                    fn encode<W: std::io::Write>(
                        &self,
                        c: libipld::cbor::DagCborCodec,
                        w: &mut W,
                    ) -> libipld::Result<()> {
                        use libipld::codec::Encode;
                        match *self {
                            Enum::Variant => Encode::encode("test", c, w)?,
                            Enum::Other => Encode::encode("Other", c, w)?,
                        };
                        Ok(())
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn encode_int_enum() {
        let e = gen_encode(&ast(quote! {
            #[derive(DagCbor)]
            #[ipld(repr = "int")]
            enum Enum {
                Variant = 1,
                Other = 0,
            }
        }))
        .to_string();
        assert_eq!(
            e,
            quote! {
                impl libipld::codec::Encode<libipld::cbor::DagCborCodec> for Enum {
                    fn encode<W: std::io::Write>(
                        &self,
                        c: libipld::cbor::DagCborCodec,
                        w: &mut W,
                    ) -> libipld::Result<()> {
                        use libipld::codec::Encode;
                        Encode::encode(&(*self as u64), c, w)
                    }
                }
            }
            .to_string()
        );
    }
}
