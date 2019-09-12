use ipld_derive::Ipld;
use libipld::{ipld, FromIpld, IpldError, ToIpld};

#[derive(Clone, Debug, Default, Ipld, PartialEq)]
#[ipld(repr = "list")]
struct ListRepr {
    a: bool,
    b: bool,
}

#[derive(Clone, Debug, Ipld, PartialEq)]
#[ipld(repr = "kinded")]
enum KindedRepr {
    A(bool),
    //B { a: u32 },
}

fn main() -> Result<(), IpldError> {
    let data = ListRepr::default();
    let ipld = data.to_ipld().to_owned();
    let expect = ipld!([false, false]);
    assert_eq!(ipld, expect);
    let data2 = ListRepr::from_ipld(ipld)?;
    assert_eq!(data, data2);

    let data = KindedRepr::A(true);
    let ipld = data.to_ipld().to_owned();
    let expect = ipld!([true]);
    assert_eq!(ipld, expect);
    let data2 = KindedRepr::from_ipld(ipld)?;
    assert_eq!(data, data2);
    
    /*let data = KindedRepr::B { a: 42 };
    let ipld = data.to_ipld().to_owned();
    let expect = ipld!({ "a": 42 });
    assert_eq!(ipld, expect);
    let data2 = KindedRepr::from_ipld(ipld)?;
    assert_eq!(data, data2);*/
    
    Ok(())
}
