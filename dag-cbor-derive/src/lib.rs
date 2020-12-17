use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure};

decl_derive!([DagCbor, attributes(ipld)] => dag_cbor_derive);

mod ast;
mod attr;
mod gen;
mod parse;

fn dag_cbor_derive(s: Structure) -> TokenStream {
    let ast = parse::parse(&s);
    let encode = gen::gen_encode(&ast);
    let decode = gen::gen_decode(&ast);
    let try_read_cbor = gen::gen_try_read_cbor(&ast);
    quote! {
        #encode
        #decode
        #try_read_cbor
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let t = trybuild::TestCases::new();
        t.pass("examples/basic.rs");
        t.pass("examples/name_attr.rs");
        t.pass("examples/repr_attr.rs");
    }
}
