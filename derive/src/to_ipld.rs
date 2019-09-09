use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

pub fn to_ipld(ident: &Ident, data: &Data) -> TokenStream {
    let inner = match data {
        Data::Struct(data) => from_struct(ident, data),
        Data::Enum(data) => from_enum(ident, data),
        Data::Union(data) => from_union(ident, data),
    };
    quote! {
        fn to_ipld<'a>(&'a self) -> libipld::IpldRef<'a> {
            #inner
        }
    }
}

fn from_struct(ident: &Ident, data: &DataStruct) -> TokenStream {
    let (matches, returns) = from_fields(quote!(#ident), &data.fields);
    quote! {
        match self {
            #matches => #returns
        }
    }
}

fn from_enum(ident: &Ident, data: &DataEnum) -> TokenStream {
    let vars: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|var| {
            let var_ident = &var.ident;
            let name = var_ident.to_string();
            let (matches, returns) = from_fields(quote!(#ident::#var_ident), &var.fields);
            quote! {
                #matches => {
                    let mut tmpmap = std::collections::BTreeMap::new();
                    tmpmap.insert(libipld::IpldKey::from(#name), #returns);
                    libipld::IpldRef::OwnedMap(tmpmap)
                }
            }
        })
        .collect();
    quote! {
        match self {
            #(#vars),*
        }
    }
}

fn from_union(_ident: &Ident, _input: &DataUnion) -> TokenStream {
    panic!("Unions not supported.");
}

fn from_fields(ident: TokenStream, fields: &Fields) -> (TokenStream, TokenStream) {
    match fields {
        Fields::Named(fields) => {
            let idents: Vec<Ident> = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap().to_owned())
                .collect();
            let ipld: Vec<TokenStream> = idents
                .iter()
                .map(|ident| {
                    let name = ident.to_string();
                    quote!(tmpmap.insert(libipld::IpldKey::from(#name), libipld::IpldRef::from(#ident));)
                })
                .collect();
            (
                quote!(#ident { #(#idents),* }),
                quote!({
                    let mut tmpmap = std::collections::BTreeMap::new();
                    #(#ipld)*
                    libipld::IpldRef::OwnedMap(tmpmap)
                })
            )
        }
        Fields::Unnamed(fields) => {
            let len = fields.unnamed.len();
            let idents: Vec<Ident> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let mut name = "__self_".to_string();
                    name.push_str(&i.to_string());
                    Ident::new(&name, Span::call_site())
                })
                .collect();
            let ipld: Vec<TokenStream> = idents
                .iter()
                .map(|ident| quote!(tmplist.push(libipld::IpldRef::from(#ident));))
                .collect();
            (
                quote!(#ident(#(#idents),*)),
                quote!({
                    let mut tmplist = Vec::with_capacity(#len);
                    #(#ipld)*
                    libipld::IpldRef::OwnedList(tmplist)
                })
            )
        }
        Fields::Unit => (quote!(#ident), quote!(libipld::IpldRef::Null)),
    }
}
