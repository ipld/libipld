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
        fn from_ipld(mut ipld: libipld::Ipld) -> Result<Self, failure::Error> {
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
            return Err(failure::format_err!("Expected map."));
        };
        #(#vars)*
        Err(failure::format_err!("No variant matched."))
    }
}

fn from_union(_ident: &Ident, _data: &DataUnion) -> TokenStream {
    quote! {
        Err(failure::format_err!("Unions not supported."))
    }
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
                            return Err(failure::format_err!("Expected key #name"));
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
                    return Err(failure::format_err!("Expected map."));
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
                        return Err(failure::format_err!("List has wrong length."));
                    }
                    Ok(#ident(#(#fields),*))
                } else {
                    return Err(failure::format_err!("Expected list."));
                }
            }
        }
        Fields::Unit => quote!(Ok(#ident)),
    }
}
