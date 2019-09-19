use async_std::task;
use dag_cbor_derive::DagCbor;
use failure::Error;
use libipld::codec::cbor::{ReadCbor, WriteCbor};
use libipld::{ipld, Codec, DagCborCodec};

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
struct RenameFields {
    #[ipld(name = "hashAlg")]
    hash_alg: String,
}

async fn run() -> Result<(), Error> {
    let data = RenameFields {
        hash_alg: "murmur3".to_string(),
    };
    let mut bytes = Vec::new();
    data.write_cbor(&mut bytes).await?;
    let ipld = DagCborCodec::decode(&bytes).await?;
    let expect = ipld!({
        "hashAlg": "murmur3",
    });
    assert_eq!(ipld, expect);
    let data2 = RenameFields::read_cbor(&mut bytes.as_slice()).await?;
    assert_eq!(data, data2);
    Ok(())
}

fn main() -> Result<(), Error> {
    task::block_on(run())
}
