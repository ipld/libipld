use async_std::task;
use dag_cbor::{Codec, DagCborCodec, ReadCbor, WriteCbor};
use dag_cbor_derive::DagCbor;
use failure::Error;
use libipld_macro::ipld;

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
        data.write_cbor(&mut bytes).await?;
        let ipld = DagCborCodec::decode(&bytes).await?;
        assert_eq!(ipld, $ipld);
        let data = <$ty>::read_cbor(&mut bytes.as_slice()).await?;
        assert_eq!(data, $data);
    };
}

async fn run() -> Result<(), Error> {
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

fn main() -> Result<(), Error> {
    task::block_on(run())
}
