use async_std::task;
use dag_cbor_derive::DagCbor;
use failure::Error;
use libipld::codec::cbor::{ReadCbor, WriteCbor};
use libipld::{ipld, Codec, DagCborCodec, Ipld};
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

macro_rules! test_case {
    ($data:expr, $ty:ty, $ipld:expr) => {
        let data = $data;
        let mut bytes = Vec::new();
        data.write_cbor(&mut bytes).await?;
        let ipld = DagCborCodec::decode(&bytes).await?;
        assert_eq!(ipld, $ipld);
        let data = <$ty>::read_cbor(&mut bytes.as_slice()).await?;
        assert_eq!(data, $data);
    };
}

async fn run() -> Result<(), Error> {
    test_case! {
        NamedStruct::default(),
        NamedStruct,
        ipld!({
            "boolean": false,
            "integer": 0,
            "float": 0.0,
            "string": "",
            "bytes": [],
            "list": [],
            "map": {},
        })
    }

    test_case! {
        TupleStruct::default(),
        TupleStruct,
        ipld!([false, 0])
    }

    test_case! {
        UnitStruct::default(),
        UnitStruct,
        ipld!([])
    }

    test_case! {
        Enum::A,
        Enum,
        ipld!({ "A": [] })
    }

    test_case! {
        Enum::B(true, 42),
        Enum,
        ipld!({ "B": [true, 42] })
    }

    test_case! {
        Enum::C { boolean: true, int: 42 },
        Enum,
        ipld!({ "C": { "boolean": true, "int": 42} })
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    task::block_on(run())
}
