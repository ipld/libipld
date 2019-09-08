use ipld_derive::Ipld;
use cid::Cid;

#[derive(Ipld)]
struct BasicStruct {
    boolean: bool,
    integer: u32,
    float: f64,
    string: String,
    bytes: Vec<u8>,
    link: Cid,
}

fn main() {}
