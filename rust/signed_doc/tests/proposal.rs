//! Integration test for proposal document validation part.

use catalyst_signed_doc::{providers::tests::TestCatalystSignedDocumentProvider, *};
use catalyst_types::id_uri::role_index::RoleIndex;

mod common;

#[tokio::test]
async fn test_valid_proposal_doc() {
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(doc_types::PROPOSAL_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        Some(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
        })),
        RoleIndex::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_proposal_doc_with_empty_provider() {
    // dummy template doc to dummy provider
    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        Some(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
        })),
        RoleIndex::Proposer,
    )
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc() {
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        Some(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            // without specifying template id
            "template": serde_json::Value::Null,
        })),
        RoleIndex::Proposer,
    )
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
