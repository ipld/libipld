//! The purpose of this example is to test whether the derive compiles if the libipld package was
//! imported from within this repo as libipld_core
use libipld_cbor;
use libipld_cbor_derive::DagCborInternal;

#[derive(Clone, DagCborInternal, Debug, Default, PartialEq)]
struct NamedStruct {
    boolean: bool,
    integer: u32,
    float: f64,
    string: String,
}
