use libipld::cbor::DagCbor;
use libipld::codec::{Decode, Encode};
use libipld::ipld::Ipld;
use libipld::{ipld, DagCbor};

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
    B { a: u32 },
}

macro_rules! test_case {
    ($data:expr, $ty:ty, $ipld:expr) => {
        let data = $data;
        let mut bytes = Vec::new();
        data.encode(&mut bytes)?;
        let ipld: Ipld = Decode::<DagCbor>::decode(&mut bytes.as_slice())?;
        assert_eq!(ipld, $ipld);
        let data: $ty = Decode::<DagCbor>::decode(&mut bytes.as_slice())?;
        assert_eq!(data, $data);
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_case! {
        ListRepr::default(),
        ListRepr,
        ipld!([false, false])
    }

    test_case! {
        KindedRepr::A(true),
        KindedRepr,
        ipld!([true])
    }

    test_case! {
        KindedRepr::B { a: 42 },
        KindedRepr,
        ipld!({ "a": 42 })
    }

    Ok(())
}
