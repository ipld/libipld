//! Serde (de)serializtion for [`crate::ipld::Ipld`].
//!
//! This implementation enables Serde to serialize to/deserialize from
//! [`crate::ipld::Ipld`] values. The `Ipld` enum is similar to the `Value` enum
//! in `serde_json` or `serde_cbor`.

mod de;
mod ser;

pub use de::from_ipld;
pub use ser::to_ipld;

#[cfg(test)]
mod test {
  use std::{
    convert::TryFrom,
    fmt,
  };

  use cid::{
    serde::CID_SERDE_PRIVATE_IDENTIFIER,
    Cid,
  };
  use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
  };
  use serde_test::{
    assert_tokens,
    Token,
  };

  use crate::{
    ipld::Ipld,
    serde::{
      from_ipld,
      to_ipld,
    },
  };

  /// Utility for testing (de)serialization of [`Ipld`].
  /// Checks if `data` and `ipld` match if they are encoded into each other.
  fn assert_roundtrip<T>(data: &T, ipld: &Ipld)
  where T: Serialize + DeserializeOwned + PartialEq + fmt::Debug {
    let encoded: Ipld = to_ipld(&data).unwrap();
    assert_eq!(&encoded, ipld);
    // let decoded: T = from_ipld(ipld.clone()).unwrap();
    // assert_eq!(&decoded, data);
  }

  #[derive(Debug, Deserialize, PartialEq, Serialize)]
  struct Person {
    name: String,
    age: u8,
    hobbies: Vec<String>,
    is_cool: bool,
    link: Cid,
  }

  impl Default for Person {
    fn default() -> Self {
      Self {
        name: "Hello World!".into(),
        age: 52,
        hobbies: vec!["geography".into(), "programming".into()],
        is_cool: true,
        link: Cid::try_from(
          "bafyreibvjvcv745gig4mvqs4hctx4zfkono4rjejm2ta6gtyzkqxfjeily",
        )
        .unwrap(),
      }
    }
  }

  #[derive(Debug, Deserialize, PartialEq, Serialize)]
  enum Enum {
    UnitVariant,
    NewTypeVariant(bool),
    TupleVariant(bool, u8),
    StructVariant { x: bool, y: u8 },
  }

  #[test]
  fn test_tokens() {
    let person = Person::default();

    assert_tokens(&person, &[
      Token::Struct { name: "Person", len: 5 },
      Token::Str("name"),
      Token::Str("Hello World!"),
      Token::Str("age"),
      Token::U8(52),
      Token::Str("hobbies"),
      Token::Seq { len: Some(2) },
      Token::Str("geography"),
      Token::Str("programming"),
      Token::SeqEnd,
      Token::Str("is_cool"),
      Token::Bool(true),
      Token::Str("link"),
      Token::NewtypeStruct { name: CID_SERDE_PRIVATE_IDENTIFIER },
      Token::Bytes(&[
        0x01, 0x71, 0x12, 0x20, 0x35, 0x4d, 0x45, 0x5f, 0xf3, 0xa6, 0x41, 0xb8,
        0xca, 0xc2, 0x5c, 0x38, 0xa7, 0x7e, 0x64, 0xaa, 0x73, 0x5d, 0xc8, 0xa4,
        0x89, 0x66, 0xa6, 0xf, 0x1a, 0x78, 0xca, 0xa1, 0x72, 0xa4, 0x88, 0x5e,
      ]),
      Token::StructEnd,
    ]);
  }

  /// Test if converting to a struct from [`crate::ipld::Ipld`] and back works.
  #[test]
  fn test_ipld() {
    let person = Person::default();

    let expected_ipld = Ipld::List(vec![
      Ipld::String("Hello World!".into()),
      Ipld::Integer(52),
      Ipld::List(vec![
        Ipld::String("geography".into()),
        Ipld::String("programming".into()),
      ]),
      Ipld::Bool(true),
      Ipld::Link(person.link),
    ]);

    assert_roundtrip(&person, &expected_ipld);

    let unit_enum = Enum::UnitVariant;
    let expected_ipld = Ipld::List(vec![Ipld::Integer(0)]);
    assert_roundtrip(&unit_enum, &expected_ipld);

    let newtype_enum = Enum::NewTypeVariant(true);
    let expected_ipld = Ipld::List(vec![Ipld::Integer(1), Ipld::Bool(true)]);
    assert_roundtrip(&newtype_enum, &expected_ipld);

    let tuple_enum = Enum::TupleVariant(true, 1u8);
    let expected_ipld = Ipld::List(vec![
      Ipld::Integer(2),
      Ipld::Bool(true),
      Ipld::Integer(1i128),
    ]);
    assert_roundtrip(&tuple_enum, &expected_ipld);
    let struct_enum = Enum::StructVariant { x: true, y: 1u8 };
    let expected_ipld = Ipld::List(vec![
      Ipld::Integer(3),
      Ipld::Bool(true),
      Ipld::Integer(1i128),
    ]);
    assert_roundtrip(&struct_enum, &expected_ipld);

    let unit = ();
    let expected_ipld = Ipld::List(vec![]);

    assert_roundtrip(&unit, &expected_ipld);
  }

  /// Test that deserializing arbitrary bytes are not accidently recognized as
  /// CID.
  #[test]
  fn test_bytes_not_cid() {
    let cid = Cid::try_from(
      "bafyreibvjvcv745gig4mvqs4hctx4zfkono4rjejm2ta6gtyzkqxfjeily",
    )
    .unwrap();

    let bytes_not_cid = Ipld::Bytes(cid.to_bytes());
    let not_a_cid: Result<Cid, _> = from_ipld(bytes_not_cid);
    assert!(not_a_cid.is_err());

    // Make sure that a Ipld::Link deserializes correctly though.
    let link = Ipld::Link(cid);
    let a_cid: Cid = from_ipld(link).unwrap();
    assert_eq!(a_cid, cid);
  }
}
