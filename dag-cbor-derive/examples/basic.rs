//use cid::Cid;
use dag_cbor_derive::DagCbor;
use libipld::{Ipld, Result, codec::cbor::WriteCbor};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
struct NamedStruct {
    boolean: bool,
    integer: u32,
    float: f64,
    string: String,
    bytes: Vec<u8>,
    list: Vec<Ipld>,
    map: BTreeMap<String, Ipld>,
    //link: Cid,
}

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
struct TupleStruct(bool, u32);

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
struct UnitStruct;

#[derive(Clone, Debug, PartialEq, DagCbor)]
enum Enum {
    A,
    B(bool, u32),
    C { boolean: bool, int: u32 },
}

#[derive(Clone, Debug, PartialEq, DagCbor)]
struct Nested {
    ipld: Ipld,
    list_of_derived: Vec<Enum>,
    map_of_derived: BTreeMap<String, NamedStruct>,
}

fn main() -> Result<()> {
    let mut bytes = Vec::new();
    let data = NamedStruct::default();
    data.write_cbor(&mut bytes)?;
    /*let data2 = NamedStruct::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    let mut bytes = Vec::new();
    let data = TupleStruct::default();
    data.write_cbor(&mut bytes)?;
    /*let data2 = TupleStruct::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    let mut bytes = Vec::new();
    let data = UnitStruct::default();
    data.write_cbor(&mut bytes)?;
    /*let data2 = UnitStruct::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    let mut bytes = Vec::new();
    let data = Enum::A;
    data.write_cbor(&mut bytes)?;
    /*let data2 = Enum::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    let mut bytes = Vec::new();
    let data = Enum::B(true, 42);
    data.write_cbor(&mut bytes)?;
    /*let data2 = Enum::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    let mut bytes = Vec::new();
    let data = Enum::C {
        boolean: true,
        int: 42,
    };
    data.write_cbor(&mut bytes)?;
    /*let data2 = Enum::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    Ok(())
}
