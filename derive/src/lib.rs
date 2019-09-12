use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure};

decl_derive!([Ipld, attributes(ipld)] => ipld_derive);

mod gen;

fn ipld_derive(s: Structure) -> TokenStream {
    let to_ipld = gen::to_ipld(&s);
    let from_ipld = gen::from_ipld(&s);
    s.gen_impl(quote! {
        use core::convert::TryInto;
        use core::result::Result;
        use libipld::{Ipld, IpldError, IpldRef, ToIpld, FromIpld};
        use std::collections::BTreeMap;

        gen impl ToIpld for @Self {
            #to_ipld
        }

        gen impl FromIpld for @Self {
            #from_ipld
        }
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
