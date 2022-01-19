use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure};

decl_derive!([DeserializeIpld, attributes(ipld)] => deserialize_derive);

mod ast;
mod attr;
mod gen;
mod parse;

fn deserialize_derive(s: Structure) -> TokenStream {
    let ast = parse::parse(&s);
    let deserialize = gen::gen_deserialize(&ast);

    quote! {
        #deserialize
    }
}
