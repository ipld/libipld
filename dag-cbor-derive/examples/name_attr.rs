use dag_cbor_derive::DagCbor;
use libipld::{Codec, DagCborCodec, ipld, Result};
use libipld::codec::cbor::WriteCbor;

#[derive(Clone, Debug, Default, PartialEq, DagCbor)]
struct RenameFields {
    #[ipld(name = "hashAlg")]
    hash_alg: String,
}

fn main() -> Result<()> {
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
    /*let data2 = RenameFields::from_ipld(ipld)?;
    assert_eq!(data, data2);*/
    Ok(())
}
