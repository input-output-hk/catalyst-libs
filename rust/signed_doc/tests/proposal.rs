//! Integration test for proposal document validation part.

use catalyst_signed_doc::*;

mod common;

#[tokio::test]
async fn test_valid_proposal_doc() {
    let (template_doc, template_doc_id) =
        common::create_dummy_doc(doc_types::PROPOSAL_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "template": {
          "id": template_doc_id
        },
    })))
    .unwrap();

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
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "template": {
          "id": template_doc_id
        },
    })))
    .unwrap();

    let provider = common::DummyCatSignDocProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc() {
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "alg": Algorithm::EdDSA.to_string(),
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        // without specifying template id
        "template": serde_json::Value::Null,
    })))
    .unwrap();

    let provider = common::DummyCatSignDocProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
