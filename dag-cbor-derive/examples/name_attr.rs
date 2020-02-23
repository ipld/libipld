use dag_cbor::{Codec, DagCborCodec, ReadCbor, WriteCbor};
use dag_cbor_derive::DagCbor;
use failure::Error;
use libipld_macro::ipld;

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
struct RenameFields {
    #[ipld(name = "hashAlg")]
    hash_alg: String,
}

fn run() -> Result<(), Error> {
    let data = RenameFields {
        hash_alg: "murmur3".to_string(),
    };
    let mut bytes = Vec::new();
    data.write_cbor(&mut bytes)?;
    let ipld = DagCborCodec::decode(&bytes)?;
    let expect = ipld!({
        "hashAlg": "murmur3",
    });
    assert_eq!(ipld, expect);
    let data2 = RenameFields::read_cbor(&mut bytes.as_slice())?;
    assert_eq!(data, data2);
    Ok(())
}

fn main() -> Result<(), Error> {
    run()
}
