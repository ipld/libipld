use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

pub fn from_ipld(ident: &Ident, data: &Data) -> TokenStream {
    let inner = match data {
        Data::Struct(data) => from_struct(ident, data),
        Data::Enum(data) => from_enum(ident, data),
        Data::Union(data) => from_union(ident, data),
    };
    quote! {
        fn from_ipld(mut ipld: libipld::Ipld) -> core::result::Result<Self, libipld::IpldError> {
            use core::convert::TryInto;
            #inner
        }
    }
}

fn from_struct(ident: &Ident, data: &DataStruct) -> TokenStream {
    from_fields(quote!(#ident), &data.fields)
}

fn from_enum(ident: &Ident, data: &DataEnum) -> TokenStream {
    let vars: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|var| {
            let var_ident = &var.ident;
            let name = var_ident.to_string();
            let fields = from_fields(quote!(#ident::#var_ident), &var.fields);
            quote! {
                if let Some(ipld) = map.get_mut(&#name.into()) {
                    return #fields;
                }
            }
        })
        .collect();
    quote! {
        let map = if let Ipld::Map(ref mut map) = ipld {
            map
        } else {
            return Err(libipld::IpldError::NotMap);
        };
        #(#vars)*
        Err(libipld::IpldError::KeyNotFound)
    }
}

fn from_union(_ident: &Ident, _data: &DataUnion) -> TokenStream {
    panic!("Unions not supported.");
}

fn from_fields(ident: TokenStream, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let fields: Vec<TokenStream> = fields
                .named
                .iter()
                .map(|field| {
                    let ident = field.ident.as_ref().unwrap().to_owned();
                    let name = ident.to_string();
                    quote! {
                        #ident: if let Some(ipld) = map.remove(&#name.into()) {
                            ipld.try_into()?
                        } else {
                            return Err(libipld::IpldError::KeyNotFound);
                        }
                    }
                })
                .collect();
            quote! {
                if let Ipld::Map(ref mut map) = ipld {
                    Ok(#ident {
                        #(#fields),*
                    })
                } else {
                    return Err(libipld::IpldError::NotMap);
                }
            }
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len();
            let fields: Vec<TokenStream> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    quote! {
                        list[#i].clone().try_into()?
                    }
                })
                .collect();
            quote! {
                if let Ipld::List(list) = ipld {
                    if list.len() != #len {
                        return Err(libipld::IpldError::IndexNotFound);
                    }
                    Ok(#ident(#(#fields),*))
                } else {
                    return Err(libipld::IpldError::NotList);
                }
            }
        }
        Fields::Unit => quote!(Ok(#ident)),
    }
}
