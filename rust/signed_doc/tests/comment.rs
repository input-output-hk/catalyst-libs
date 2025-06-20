//! Integration test for comment document validation part.

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

#[tokio::test]
async fn test_valid_comment_doc() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(deprecated::COMMENT_TEMPLATE_UUID_TYPE).unwrap();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
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
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_comment_doc_old_type() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(deprecated::COMMENT_TEMPLATE_UUID_TYPE).unwrap();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            // Using old (single uuid)
            "type": deprecated::COMMENT_DOCUMENT_UUID_TYPE,
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
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_comment_doc_with_reply() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(deprecated::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let comment_doc_id = UuidV7::new();
    let comment_doc_ver = UuidV7::new();
    let comment_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": comment_doc_id,
            "ver": comment_doc_ver,
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "template": { "id": template_doc_id.to_string(), "ver": template_doc_ver.to_string() },
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
           "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
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
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();
    provider.add_document(comment_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_invalid_comment_doc() {
    let (proposal_doc, ..) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(deprecated::COMMENT_TEMPLATE_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            // without ref
            "ref": serde_json::Value::Null
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(template_doc).unwrap();
    provider.add_document(proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
