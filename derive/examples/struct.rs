//use cid::Cid;
use ipld_derive::Ipld;
use libipld::{Ipld, IpldError, IpldKey};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Ipld, PartialEq)]
struct NamedStruct {
    boolean: bool,
    integer: u32,
    float: f64,
    string: String,
    bytes: Vec<u8>,
    list: Vec<Ipld>,
    map: BTreeMap<IpldKey, Ipld>,
    //link: Cid,
}

#[derive(Clone, Debug, Default, Ipld, PartialEq)]
struct TupleStruct(bool, u32);

#[derive(Clone, Debug, Default, Ipld, PartialEq)]
struct UnitStruct;

#[derive(Clone, Debug, Ipld, PartialEq)]
enum Enum {
    A,
    B(bool, u32),
    C { boolean: bool, int: u32 },
}

fn main() -> Result<(), IpldError> {
    let data = NamedStruct::default();
    let ipld = data.to_ipld().to_owned();
    println!("{:?}", ipld);
    let data2 = NamedStruct::from_ipld(ipld)?;
    assert_eq!(data, data2);

    let data = TupleStruct::default();
    let ipld = data.to_ipld().to_owned();
    println!("{:?}", ipld);
    let data2 = TupleStruct::from_ipld(ipld)?;
    assert_eq!(data, data2);

    let data = UnitStruct::default();
    let ipld = data.to_ipld().to_owned();
    println!("{:?}", ipld);
    let data2 = UnitStruct::from_ipld(ipld)?;
    assert_eq!(data, data2);

    let data = Enum::A;
    let ipld = data.to_ipld().to_owned();
    println!("{:?}", ipld);
    let data2 = Enum::from_ipld(ipld)?;
    assert_eq!(data, data2);

    let data = Enum::B(true, 42);
    let ipld = data.to_ipld().to_owned();
    println!("{:?}", ipld);
    let data2 = Enum::from_ipld(ipld)?;
    assert_eq!(data, data2);

    let data = Enum::C {
        boolean: true,
        int: 42,
    };
    let ipld = data.to_ipld().to_owned();
    println!("{:?}", ipld);
    let data2 = Enum::from_ipld(ipld)?;
    assert_eq!(data, data2);

    Ok(())
}
