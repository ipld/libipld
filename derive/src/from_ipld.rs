use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;

pub fn from_ipld(_input: &Data) -> TokenStream {
    quote! {
        fn from_ipld(ipld: Ipld) /*-> Self*/ {
            /*Self {

            }*/
        }
    }
}
