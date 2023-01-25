#![deny(warnings)]

use anyhow::anyhow;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, ToTokens};
use synstructure::{decl_derive, Structure};

decl_derive!([DagCbor, attributes(ipld)] => dag_cbor_derive);

decl_derive!([DagCborInternal, attributes(ipld)] => dag_cbor_derive_internal);

mod ast;
mod attr;
mod gen;
mod parse;

// Entry point for the DagCbor derive macro
fn dag_cbor_derive(s: Structure) -> TokenStream {
    let libipld_core = match use_crate("libipld") {
        Ok(ident) => ident,
        Err(error) => return error,
    };
    let libipld_cbor = quote!(#libipld_core::cbor);
    let ast = parse::parse(&s);
    let encode = gen::gen_encode(&ast, &libipld_core, &libipld_cbor);
    let decode = gen::gen_decode(&ast, &libipld_core, &libipld_cbor);
    quote! {
        #encode
        #decode
    }
}

// Entry point for the DagCborCrate derive macro
// This variant of the macro may be used within libipld itself
// as it uses the API exposed by the sub-crates within the workspace directly
// instead of the API exposed by the top level libipld crate.
fn dag_cbor_derive_internal(s: Structure) -> TokenStream {
    let libipld_core = quote!(libipld_core);
    let libipld_cbor = quote!(libipld_cbor);

    let ast = parse::parse(&s);
    let encode = gen::gen_encode(&ast, &libipld_core, &libipld_cbor);
    let decode = gen::gen_decode(&ast, &libipld_core, &libipld_cbor);
    quote! {
        #encode
        #decode
    }
}

/// Get the name of a crate based on its original name.
///
/// This works even if the crate was renamed in the `Cargo.toml` file. If the crate is not a
/// dependency, it will lead to a compile-time error.
fn use_crate(name: &str) -> Result<TokenStream, TokenStream> {
    match crate_name(name) {
        Ok(FoundCrate::Name(n)) => Ok(syn::Ident::new(&n, Span::call_site()).to_token_stream()),
        Ok(FoundCrate::Itself) => Err(syn::Error::new(
            Span::call_site(),
            anyhow!("unsupported use of dag-cbor-derive macro from within libipld crate"),
        )
        .to_compile_error()),
        Err(err) => Err(syn::Error::new(Span::call_site(), err).to_compile_error()),
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
