use std::collections::BTreeMap;

use libipld_core::ipld::Ipld;
use libipld_core::serde::assert_roundtrip;
use libipld_derive::DeserializeIpld;
use serde::{Deserialize, Serialize};

//#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
//#[ipld(repr = "keyed")]
//pub enum Keyed {
//    A,
//    #[ipld(rename = "b")]
//    #[ipld(repr = "value")]
//    B(bool),
//    #[ipld(repr = "value")]
//    C {
//        n: u32,
//    },
//    D(bool),
//    E {
//        boolean: bool,
//    },
//}
//
//#[test]
//fn union_keyed() {
//    assert_roundtrip(DagCborCodec, &Keyed::A, &ipld!({ "A": null }));
//    assert_roundtrip(DagCborCodec, &Keyed::B(true), &ipld!({"b": true}));
//    assert_roundtrip(DagCborCodec, &Keyed::B(false), &ipld!({"b": false}));
//    assert_roundtrip(DagCborCodec, &Keyed::C { n: 1 }, &ipld!({"C": 1}));
//    assert_roundtrip(DagCborCodec, &Keyed::D(true), &ipld!({"D": [true]}));
//    assert_roundtrip(
//        DagCborCodec,
//        &Keyed::E { boolean: true },
//        &ipld!({"E": { "boolean": true }}),
//    );
//}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct MyNewtypeStruct(u16);

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct MyStruct {
    x: u16,
    y: u16,
}

#[derive(Debug, Eq, DeserializeIpld, PartialEq, Serialize)]
#[ipld(repr = "kinded")]
#[serde(untagged)]
enum Kinded {
    A,
    B(bool),
    C(u8, String),
    D { boolean: bool },
    E { n: u32, p: bool },
    F(MyNewtypeStruct),
    G((u8, u8, u8)),
    H(MyStruct),
}

#[test]
fn union_kinded() {
    assert_roundtrip(&Kinded::A, &Ipld::Null);
    assert_roundtrip(&Kinded::B(false), &Ipld::Bool(false));
    assert_roundtrip(
        &Kinded::C(27, "hello world!".into()),
        &Ipld::List(vec![Ipld::Integer(27), Ipld::String("hello world!".into())]),
    );
    assert_roundtrip(
        &Kinded::D { boolean: true },
        &Ipld::Map(BTreeMap::from([("boolean".into(), Ipld::Bool(true))])),
    );
    assert_roundtrip(
        &Kinded::E { n: 5, p: true },
        &Ipld::Map(BTreeMap::from([
            ("n".into(), Ipld::Integer(5)),
            ("p".into(), Ipld::Bool(true)),
        ])),
    );
    assert_roundtrip(&Kinded::F(MyNewtypeStruct(342)), &Ipld::Integer(342));
    assert_roundtrip(
        &Kinded::G((44, 55, 66)),
        &Ipld::List(vec![
            Ipld::Integer(44),
            Ipld::Integer(55),
            Ipld::Integer(66),
        ]),
    );
    assert_roundtrip(
        &Kinded::H(MyStruct { x: 42, y: 24 }),
        &Ipld::Map(BTreeMap::from([
            ("x".into(), Ipld::Integer(42)),
            ("y".into(), Ipld::Integer(24)),
        ])),
    );
}

//#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
//#[ipld(repr = "int-tuple")]
//pub enum IntTuple {
//    A,
//    #[ipld(rename = "b")]
//    #[ipld(repr = "value")]
//    B(bool),
//    #[ipld(repr = "value")]
//    C {
//        n: u32,
//    },
//    D(bool),
//    E {
//        boolean: bool,
//    },
//}
//
//#[test]
//fn union_int_tuple() {
//    assert_roundtrip(DagCborCodec, &IntTuple::A, &ipld!([0, null]));
//    assert_roundtrip(DagCborCodec, &IntTuple::B(true), &ipld!([1, true]));
//    assert_roundtrip(DagCborCodec, &IntTuple::B(false), &ipld!([1, false]));
//    assert_roundtrip(DagCborCodec, &IntTuple::C { n: 1 }, &ipld!([2, 1]));
//    assert_roundtrip(DagCborCodec, &IntTuple::D(true), &ipld!([3, [true]]));
//    assert_roundtrip(
//        DagCborCodec,
//        &IntTuple::E { boolean: true },
//        &ipld!([4, { "boolean": true }]),
//    );
//}
