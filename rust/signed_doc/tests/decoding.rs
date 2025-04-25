//! Integration test for COSE decoding part.

use catalyst_signed_doc::*;
use catalyst_types::id_uri::role_index::RoleIndex;
use common::create_dummy_key_pair;
use ed25519_dalek::ed25519::signature::Signer;

mod common;

#[test]
fn catalyst_signed_doc_cbor_roundtrip_test() {
    let (uuid_v7, uuid_v4, metadata_fields) = common::test_metadata();
    let (sk, _, kid) = create_dummy_key_pair(RoleIndex::ROLE_0).unwrap();

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();

    assert!(!doc.problem_report().is_problematic());

    let bytes: Vec<u8> = doc.try_into().unwrap();
    let decoded: CatalystSignedDocument = bytes.as_slice().try_into().unwrap();
    let extra_fields: ExtraFields = serde_json::from_value(metadata_fields).unwrap();

    assert_eq!(decoded.doc_type().unwrap(), uuid_v4);
    assert_eq!(decoded.doc_id().unwrap(), uuid_v7);
    assert_eq!(decoded.doc_ver().unwrap(), uuid_v7);
    assert_eq!(decoded.doc_content().decoded_bytes().unwrap(), &content);
    assert_eq!(decoded.doc_meta(), &extra_fields);
}

#[test]
fn catalyst_signed_doc_cbor_roundtrip_kid_as_id_test() {
    let (_, _, metadata_fields) = common::test_metadata();
    let (sk, _, kid) = create_dummy_key_pair(RoleIndex::ROLE_0).unwrap();
    // transform Catalyst ID URI form to the ID form
    let kid = kid.as_id();

    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
        .add_signature(|m| sk.sign(&m).to_vec(), &kid)
        .unwrap()
        .build();

    assert!(doc.problem_report().is_problematic());
}
