#![deny(missing_docs)]
#![deny(warnings)]

use anyhow::Result;
use assert_json_diff::assert_json_eq;
use once_cell::sync::Lazy;
use std::{io::Cursor, path::PathBuf, sync::Mutex};
use testmark::{Document, Hunk};

use libipld_core::codec::{Decode, Encode};
use libipld_json::DagJsonCodec;

use libipld_jose::*;

// Load the fixtures file once
static FIXTURES: Lazy<Mutex<Document>> = Lazy::new(|| {
    let fpath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("dag-jose.md");
    Document::from_file(&fpath)
        .expect("fixture file dag-jose.md should be a markdown file")
        .into()
});

// Find hunks of data
trait HunkFinder<'a>: Sized {
    fn find_hunk(self, name: &str) -> Option<&'a Hunk>;
    fn must_find_hunk(self, name: &str) -> &'a Hunk {
        self.find_hunk(name)
            .expect(format!("fixture should have hunk: {}", name).as_str())
    }
}
// Implement hunk finder for a Document
impl<'a> HunkFinder<'a> for &'a Document {
    fn find_hunk(self, name: &str) -> Option<&'a Hunk> {
        self.hunks().iter().find(|h| h.name() == name)
    }
}

// remove whitespace from anywhere inside of an utf-8 encoded byte slice.
fn remove_whitespace(data: &[u8]) -> Result<String> {
    let s = String::from_utf8(data.to_vec())?;
    Ok(s.chars().filter(|c| !c.is_whitespace()).collect())
}

macro_rules! test_fixture {
    ($fname:ident,$name:expr) => {
        #[test]
        fn $fname() {
            decode_re_encode(
                concat!($name, "/serial.dag-jose.hex"),
                concat!($name, "/datamodel.dag-json.pretty"),
            )
        }
    };
}

// Decode hex data into DAG-JOSE and re-encode into DAG-JSON
// in order to compare against fixture data.
fn decode_re_encode(hex_name: &str, json_name: &str) {
    let fixtures = match FIXTURES.lock() {
        Ok(f) => f,
        // We can ignore poisoned errors since
        // we only need read only access to the fixture and
        // any failed test will point the mutex.
        Err(poisoned) => poisoned.into_inner(),
    };
    // Decode the hex data into a DAG-JOSE value
    let dag_jose_hex = remove_whitespace(fixtures.must_find_hunk(hex_name).data())
        .expect("hex fixture data should be UTF8");
    let jose = Jose::decode(
        DagJoseCodec,
        &mut Cursor::new(
            hex::decode(&dag_jose_hex).expect("hex fixture data should be hex encoded"),
        ),
    )
    .expect("hex fixture data should represent a DAG-JOSE value");

    // Test the we can encode back to the same hex data.
    let mut encoded_bytes = Vec::new();
    jose.encode(DagJoseCodec, &mut encoded_bytes)
        .expect("encoded DAG-JOSE value should encode to DAG-CBOR");
    assert_eq!(dag_jose_hex, hex::encode(encoded_bytes));

    // Re-encode the DAG-JOSE value in DAG-JSON
    let mut bytes = Vec::new();
    jose.encode(DagJsonCodec, &mut bytes)
        .expect("decoded DAG-JOSE value should encode to DAG-CBOR");

    // Load expected JSON data
    let dag_json = fixtures.must_find_hunk(json_name);
    // Compare JSON representations are the same
    assert_json_eq!(
        serde_json::from_slice::<serde_json::Value>(dag_json.data())
            .expect("DAG-JSON data should be JSON"),
        serde_json::from_slice::<serde_json::Value>(&bytes).expect("bytes should be JSON"),
    );
}

test_fixture!(jws, "jws");
test_fixture!(jws_signature_1, "jws-signature-1");
test_fixture!(jws_signature_2, "jws-signature-2");
test_fixture!(jws_signatures, "jws-signatures");
test_fixture!(jwe_symmetric, "jwe-symmetric");
test_fixture!(jwe_asymmetric, "jwe-asymmetric");
test_fixture!(jwe_no_recipients, "jwe-no-recipients");
test_fixture!(jwe_recipient, "jwe-recipient");
test_fixture!(jwe_recipients, "jwe-recipients");
