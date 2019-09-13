use dag_cbor_derive::DagCbor;
use libipld::codec::cbor::{ReadCbor, WriteCbor};
use libipld::{ipld, Codec, DagCborCodec, Result};

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
#[ipld(repr = "list")]
struct ListRepr {
    a: bool,
    b: bool,
}

#[derive(Clone, Debug, PartialEq, DagCbor)]
#[ipld(repr = "kinded")]
enum KindedRepr {
    A(bool),
    //B { a: u32 },
}

fn main() -> Result<()> {
    let data = ListRepr::default();
    let mut bytes = Vec::new();
    data.write_cbor(&mut bytes)?;
    let ipld = DagCborCodec::decode(&bytes)?;
    let expect = ipld!([false, false]);
    assert_eq!(ipld, expect);
    let data2 = ListRepr::read_cbor(&mut bytes.as_slice())?;
    assert_eq!(data, data2);

    let data = KindedRepr::A(true);
    let mut bytes = Vec::new();
    data.write_cbor(&mut bytes)?;
    let ipld = DagCborCodec::decode(&bytes)?;
    let expect = ipld!([true]);
    assert_eq!(ipld, expect);
    let data2 = KindedRepr::read_cbor(&mut bytes.as_slice())?;
    assert_eq!(data, data2);

    /*let data = KindedRepr::B { a: 42 };
    let ipld = data.to_ipld().to_owned();
    let expect = ipld!({ "a": 42 });
    assert_eq!(ipld, expect);
    let data2 = KindedRepr::from_ipld(ipld)?;
    assert_eq!(data, data2);*/

    Ok(())
}
