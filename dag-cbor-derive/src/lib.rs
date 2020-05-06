use proc_macro2::TokenStream;
use quote::quote;
use synstructure::{decl_derive, Structure};

decl_derive!([DagCbor, attributes(ipld)] => dag_cbor_derive);

mod gen;

fn dag_cbor_derive(s: Structure) -> TokenStream {
    let encode = gen::encode(&s);
    let try_read_cbor = gen::decode(&s);
    s.gen_impl(quote! {
        use libipld::cbor::{DagCbor, Error, Result};
        use libipld::cbor::encode::write_u64;
        use libipld::cbor::decode::{read, read_u8, read_key, TryReadCbor};
        use libipld::codec::{Encode, Decode};
        use libipld::error::{TypeError, TypeErrorType};
        use std::io::{Read, Write};

        gen impl Encode<DagCbor> for @Self {
            #encode
        }

        gen impl TryReadCbor for @Self {
            #try_read_cbor
        }

        gen impl Decode<DagCbor> for @Self {
            fn decode<R: Read>(r: &mut R) -> Result<Self> {
                read(r)
            }
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
        //t.pass("examples/repr_attr.rs");
    }
}
