use crate::cbor::*;
use crate::decode::{read_link, read_major, read_uint};
use crate::error::{UnexpectedCode, UnknownTag};
use libipld_core::error::Result;
use libipld_core::ipld::Ipld;
use std::io::SeekFrom::Current;
use std::io::{Read, Seek};

/// Validates that the supplied input is validly formed DAG-CBOR, avoiding deserialization as
/// much as possible.
pub fn validate<R: Read + Seek>(r: &mut R) -> Result<()> {
    let major = read_major(r)?;
    let skip: u64 = match major.kind() {
        // Integer: we need to consume the value to validate that the minimal
        // encoding constraint is respected.
        MajorKind::UnsignedInt | MajorKind::NegativeInt => {
            read_uint(r, major)?;
            0
        }
        // Bytes, Text, Array: opaque sized items, skip over them.
        MajorKind::ByteString | MajorKind::TextString => read_uint(r, major)?,
        MajorKind::Array => {
            validate_list(r, major)?;
            0
        }
        MajorKind::Map => {
            validate_map(r, major)?;
            0
        }
        // For tags, we only accept 42 (IPLD CID) and we validate the CID is well-formed,
        // thus having to consume the full CID.
        MajorKind::Tag => {
            let value = read_uint(r, major)?;
            if value != 42 {
                return Err(UnknownTag(value).into());
            }
            read_link(r)?;
            0
        }
        MajorKind::Other => match major {
            TRUE | FALSE | NULL => 0,
            F32 => 4,
            F64 => 8,
            m => return Err(UnexpectedCode::new::<Ipld>(m.into()).into()),
        },
    };
    r.seek(Current(skip as i64))?;
    Ok(())
}

/// Validates that a CBOR map meets DAG-CBOR constraints, i.e. String keys and DAG-CBOR values.
fn validate_map<R: Read + Seek>(r: &mut R, major: Major) -> Result<()> {
    let len = read_uint(r, major)?;

    (0..len).try_for_each(|_| {
        // Key is string.
        let major = read_major(r)?;
        if major.kind() == MajorKind::TextString {
            let len = read_uint(r, major)?;
            r.seek(Current(len as i64))?;
        } else {
            return Err(UnexpectedCode::new::<Ipld>(major.into()).into());
        }
        // Value is valid DAG_CBOR.
        validate(r)
    })
}

/// Validates that a list is formed by valid DAG-CBOR elements.
fn validate_list<R: Read + Seek>(r: &mut R, major: Major) -> Result<()> {
    let len = read_uint(r, major)?;
    (0..len).try_for_each(|_| validate(r))
}

#[cfg(test)]
mod tests {
    use crate::validate::validate;
    use crate::DagCborCodec;
    use libipld_core::cid::Cid;
    use libipld_core::codec::Codec;
    use libipld_core::multihash::MultihashDigest;
    use libipld_macro::ipld;
    use multihash::Code;
    use std::io::Cursor;

    #[test]
    pub fn test_validate() {
        let cid = Cid::new_v1(0, Code::Blake3_256.digest(&b"cid"[..]));
        let ipld = ipld!({
            "small number": 1,
            "negative number": -100,
            "zero": 0,
            "null": null,
            "list": [true, null, false],
            "bytes": vec![0, 1, 2, 3],
            "map": { "float": 0.0, "string": "hello", "bytes": vec![1, 2, 3], "link": cid.clone() },
            "link": cid,
        });

        // Valid.
        let bytes = DagCborCodec.encode(&ipld).unwrap();
        assert_eq!((), validate(&mut Cursor::new(bytes.clone())).unwrap());

        // Invalid; truncated.
        let truncated = &bytes.clone()[..bytes.len() - 2];
        assert!(validate(&mut Cursor::new(&truncated)).is_err());

        // Invalid; garbled.
        let mut garbled = bytes.clone();
        garbled[1] = 0xff;
        garbled[3] = 0xff;
        assert!(validate(&mut Cursor::new(garbled)).is_err());
    }

    #[test]
    pub fn test_invalid_indefinite_length_map() {
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
        assert!(validate(&mut Cursor::new(bytes)).is_err());
    }
}
