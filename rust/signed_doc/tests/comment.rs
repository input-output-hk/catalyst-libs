//! Integration test for comment document validation part.

use catalyst_signed_doc::{providers::tests::TestCatalystSignedDocumentProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;

mod common;

// not going to fix this tests for this feature branch.
// its going to be covered as `cat-gateway` integration tests.
#[allow(dead_code, clippy::unwrap_used)]
async fn test_valid_comment_doc() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(doc_types::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            }
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

// not going to fix this tests for this feature branch.
// its going to be covered as `cat-gateway` integration tests.
#[allow(dead_code, clippy::unwrap_used)]
async fn test_valid_comment_doc_with_reply() {
    let empty_json = serde_json::to_vec(&serde_json::json!({})).unwrap();

    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(doc_types::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let comment_doc_id = UuidV7::new();
    let comment_doc_ver = UuidV7::new();
    let comment_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "id": comment_doc_id,
            "ver": comment_doc_ver,
            "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
            "content-type": ContentType::Json.to_string(),
            "template": { "id": template_doc_id.to_string(), "ver": template_doc_ver.to_string() },
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .with_decoded_content(empty_json.clone())
        .build();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "reply": {
                "id": comment_doc_id,
                "ver": comment_doc_ver
            }
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();
    provider.add_document(comment_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

// not going to fix this tests for this feature branch.
// its going to be covered as `cat-gateway` integration tests.
#[allow(dead_code, clippy::unwrap_used)]
async fn test_invalid_comment_doc() {
    let (proposal_doc, ..) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(doc_types::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::COMMENT_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            // without ref
            "ref": serde_json::Value::Null
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
