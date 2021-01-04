use libipld::cbor::{DagCbor, DagCborCodec};
use libipld::codec::{assert_roundtrip, Codec};
use libipld::{ipld, DagCbor};

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
#[ipld(repr = "map")]
pub struct Map {
    boolean: bool,
}

#[test]
fn struct_map() {
    assert_roundtrip(
        DagCborCodec,
        &Map { boolean: true },
        &ipld!({"boolean": true}),
    );
    assert_roundtrip(
        DagCborCodec,
        &Map { boolean: false },
        &ipld!({"boolean": false}),
    );
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
pub struct Rename {
    #[ipld(rename = "bool")]
    boolean: bool,
}

#[test]
fn struct_rename() {
    assert_roundtrip(
        DagCborCodec,
        &Rename { boolean: true },
        &ipld!({"bool": true}),
    );
    assert_roundtrip(
        DagCborCodec,
        &Rename { boolean: false },
        &ipld!({"bool": false}),
    );
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
pub struct Nullable {
    nullable: Option<bool>,
}

#[test]
fn struct_nullable() {
    assert_roundtrip(
        DagCborCodec,
        &Nullable {
            nullable: Some(true),
        },
        &ipld!({"nullable": true}),
    );
    assert_roundtrip(
        DagCborCodec,
        &Nullable {
            nullable: Some(false),
        },
        &ipld!({"nullable": false}),
    );
    assert_roundtrip(
        DagCborCodec,
        &Nullable { nullable: None },
        &ipld!({ "nullable": null }),
    );
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
pub struct Implicit {
    #[ipld(default = false)]
    default: bool,
}

#[test]
fn struct_implicit() {
    assert_roundtrip(
        DagCborCodec,
        &Implicit { default: true },
        &ipld!({"default": true}),
    );
    assert_roundtrip(DagCborCodec, &Implicit { default: false }, &ipld!({}));
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
pub struct OptionalNullable {
    #[ipld(default = None)]
    nullable: Option<bool>,
}

#[test]
fn struct_optional_nullable() {
    assert_roundtrip(
        DagCborCodec,
        &OptionalNullable {
            nullable: Some(true),
        },
        &ipld!({"nullable": true}),
    );
    assert_roundtrip(
        DagCborCodec,
        &OptionalNullable {
            nullable: Some(false),
        },
        &ipld!({"nullable": false}),
    );
    assert_roundtrip(
        DagCborCodec,
        &OptionalNullable { nullable: None },
        &ipld!({}),
    );
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
#[ipld(repr = "tuple")]
pub struct Tuple(bool);

#[test]
fn struct_tuple() {
    assert_roundtrip(DagCborCodec, &Tuple(true), &ipld!([true]));
    assert_roundtrip(DagCborCodec, &Tuple(false), &ipld!([false]));
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
pub struct TupleNullable(Option<bool>);

#[test]
fn struct_tuple_nullable() {
    assert_roundtrip(DagCborCodec, &TupleNullable(Some(true)), &ipld!([true]));
    assert_roundtrip(DagCborCodec, &TupleNullable(Some(false)), &ipld!([false]));
    assert_roundtrip(DagCborCodec, &TupleNullable(None), &ipld!([null]));
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
#[ipld(repr = "value")]
pub struct Value(bool);

#[test]
fn struct_value() {
    assert_roundtrip(DagCborCodec, &Value(true), &ipld!(true));
    assert_roundtrip(DagCborCodec, &Value(false), &ipld!(false));
}

#[derive(Clone, Copy, DagCbor, Debug, Eq, PartialEq)]
pub struct IlMap {
    #[ipld(rename = "Fun")]
    fun: bool,
    #[ipld(rename = "Amt")]
    amt: i32,
}

#[test]
fn serde_cbor_compat() {
    let bytes = [
        0xBF, // Start indefinite-length map
        0x63, // First key, UTF-8 string length 3
        0x46, 0x75, 0x6e, // "Fun"
        0xF5, // First value, true
        0x63, // Second key, UTF-8 string length 3
        0x41, 0x6d, 0x74, // "Amt"
        0x21, // Second value, -2
        0xFF, // "break"
    ];
    let il_map: IlMap = DagCborCodec.decode(&bytes).unwrap();
    assert_eq!(il_map, IlMap { fun: true, amt: -2 });
}

#[derive(DagCbor)]
pub struct Generic<T: DagCbor>(T);
