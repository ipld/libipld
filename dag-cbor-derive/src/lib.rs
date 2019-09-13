use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure};

decl_derive!([DagCbor, attributes(ipld)] => dag_cbor_derive);

mod gen;

fn dag_cbor_derive(s: Structure) -> TokenStream {
    let write_cbor = gen::write_cbor(&s);
    //let from_ipld = gen::from_ipld(&s);
    s.gen_impl(quote! {
        use libipld::{Ipld, IpldError, Result};
        use libipld::cbor::WriteCbor;
        use libipld::cbor::encode::write_u64;
        use std::io::Write;

        gen impl WriteCbor for @Self {
            #write_cbor
        }

        /*gen impl FromIpld for @Self {
            #from_ipld
        }*/
    })
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
