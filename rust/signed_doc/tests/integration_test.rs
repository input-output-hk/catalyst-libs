//! Integration test for the `catalyst_signed_doc` crate.

use catalyst_signed_doc::*;

mod common;

#[test]
fn catalyst_signed_doc_cbor_roundtrip_test() {
    let (uuid_v7, uuid_v4, metadata_fields) = common::test_metadata();
    let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

    let doc = Builder::new()
        .with_json_metadata(metadata_fields.clone())
        .unwrap()
        .with_decoded_content(content.clone())
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

#[tokio::test]
async fn signature_verification_test() {
    let (signed_doc, pk) = common::get_dummy_signed_doc(None);
    assert!(!signed_doc.problem_report().is_problematic());

    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(Err(anyhow::anyhow!("some error")))
    )
    .await
    .is_err());

    assert!(
        validator::validate_signatures(&signed_doc, &common::DummyVerifyingKeyProvider(Ok(Some(pk))))
            .await
            .unwrap()
    );

    assert!(
        !validator::validate_signatures(&signed_doc, &common::DummyVerifyingKeyProvider(Ok(None)))
            .await
            .unwrap()
    );
}
