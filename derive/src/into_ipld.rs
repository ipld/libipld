use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

pub fn into_ipld(input: &Data) -> TokenStream {
    let inner = match input {
        Data::Struct(data) => from_struct(data),
        Data::Enum(data) => from_enum(data),
        Data::Union(data) => from_union(data),
    };
    quote! {
        fn into_ipld(self) -> Ipld {
            #inner
        }
    }
}

fn from_struct(input: &DataStruct) -> TokenStream {
    quote! {
        Ipld::Null
    }
}

fn from_enum(_input: &DataEnum) -> TokenStream {
    quote! {
        Ipld::Null
    }
}

fn from_union(_input: &DataUnion) -> TokenStream {
    quote! {
        Ipld::Null
    }
}
