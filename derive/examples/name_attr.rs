use ipld_derive::Ipld;
use libipld::{ipld, FromIpld, IpldError, ToIpld};

#[derive(Clone, Debug, Default, Ipld, PartialEq)]
struct RenameFields {
    #[ipld(name = "hashAlg")]
    hash_alg: String,
}

fn main() -> Result<(), IpldError> {
    let data = RenameFields {
        hash_alg: "murmur3".to_string(),
    };
    let ipld = data.to_ipld().to_owned();
    let expect = ipld!({
        "hashAlg": "murmur3",
    });
    assert_eq!(ipld, expect);
    let data2 = RenameFields::from_ipld(ipld)?;
    assert_eq!(data, data2);
    Ok(())
}
