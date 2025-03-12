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

    assert!(validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(Ok(Some(pk)))
    )
    .await
    .unwrap());

    assert!(!validator::validate_signatures(
        &signed_doc,
        &common::DummyVerifyingKeyProvider(Ok(None))
    )
    .await
    .unwrap());
}

#[tokio::test]
async fn test_valid_proposal_doc() {
    // dummy template doc to dummy provider
    let json_content = serde_json::to_vec(&serde_json::json!({})).unwrap();
    let template_doc_id = UuidV7::new();
    let template_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "type": doc_types::PROPOSAL_TEMPLATE_UUID_TYPE,
            "content-type": ContentType::Json.to_string(),
            "template": {"id": template_doc_id.to_string() }
        }))
        .unwrap()
        .with_decoded_content(json_content.clone())
        .build();

    let uuid_v7 = UuidV7::new();
    let (doc, _) = common::get_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": serde_json::Value::Null,
        "reply": serde_json::Value::Null,
        "template": {
          "id": template_doc_id
        },
        "section": serde_json::Value::Null,
        "collabs": serde_json::Value::Array(vec![]),
        "campaign_id": serde_json::Value::Null,
        "election_id":  serde_json::Value::Null,
        "brand_id":  serde_json::Value::Null,
        "category_id": serde_json::Value::Null,
    })));

    let provider =
        common::DummyCatSignDocProvider(From::from([(template_doc_id.into(), template_doc)]));

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_proposal_doc_with_empty_provider() {
    // dummy template doc to dummy provider
    let template_doc_id = UuidV7::new();

    let uuid_v7 = UuidV7::new();
    let (doc, _) = common::get_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": serde_json::Value::Null,
        "reply": serde_json::Value::Null,
        "template": {
          "id": template_doc_id
        },
        "section": serde_json::Value::Null,
        "collabs": serde_json::Value::Array(vec![]),
        "campaign_id": serde_json::Value::Null,
        "election_id":  serde_json::Value::Null,
        "brand_id":  serde_json::Value::Null,
        "category_id": serde_json::Value::Null,
    })));

    let provider = common::DummyCatSignDocProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc() {
    let uuid_v7 = UuidV7::new();
    let (doc, _) = common::get_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": serde_json::Value::Null,
        "reply": serde_json::Value::Null,
        // without specifying template id
        "template": serde_json::Value::Null,
        "section": serde_json::Value::Null,
        "collabs": serde_json::Value::Array(vec![]),
        "campaign_id": serde_json::Value::Null,
        "election_id":  serde_json::Value::Null,
        "brand_id":  serde_json::Value::Null,
        "category_id": serde_json::Value::Null,
    })));

    let provider = common::DummyCatSignDocProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
