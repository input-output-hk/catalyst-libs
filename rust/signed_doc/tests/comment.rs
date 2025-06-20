//! Integration test for comment document validation part.

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

fn dummy_proposal_doc() -> CatalystSignedDocument {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": deprecated::PROPOSAL_DOCUMENT_UUID_TYPE,
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap()
}

fn dummy_comment_template_doc() -> CatalystSignedDocument {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": deprecated::COMMENT_TEMPLATE_UUID_TYPE,
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_valid_comment_doc() {
    let dummy_proposal_doc = dummy_proposal_doc();
    let dummy_template_doc = dummy_comment_template_doc();

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
              "id": dummy_template_doc.doc_id().unwrap(),
              "ver": dummy_template_doc.doc_ver().unwrap()
            },
            "ref": {
                "id": dummy_proposal_doc.doc_id().unwrap() ,
                "ver": dummy_proposal_doc.doc_ver().unwrap()
            }
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(dummy_template_doc).unwrap();
    provider.add_document(dummy_proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_comment_doc_old_type() {
    let dummy_proposal_doc = dummy_proposal_doc();
    let dummy_template_doc = dummy_comment_template_doc();
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
              "id": dummy_template_doc.doc_id().unwrap(),
              "ver": dummy_template_doc.doc_ver().unwrap()
            },
            "ref": {
                "id": dummy_proposal_doc.doc_id().unwrap(),
                "ver": dummy_proposal_doc.doc_ver().unwrap()
            }
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(dummy_template_doc).unwrap();
    provider.add_document(dummy_proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_comment_doc_with_reply() {
    let dummy_proposal_doc = dummy_proposal_doc();
    let dummy_template_doc = dummy_comment_template_doc();

    let comment_doc_id = UuidV7::new();
    let comment_doc_ver = UuidV7::new();
    let comment_doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": comment_doc_id,
            "ver": comment_doc_ver,
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "template": {
                "id": dummy_template_doc.doc_id().unwrap(),
                "ver": dummy_template_doc.doc_ver().unwrap()
            },
            "ref": {
                "id": dummy_proposal_doc.doc_id().unwrap(),
                "ver": dummy_proposal_doc.doc_ver().unwrap()
            },
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
           "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
                "id": dummy_template_doc.doc_id().unwrap(),
                "ver": dummy_template_doc.doc_ver().unwrap()
            },
            "ref": {
                "id": dummy_proposal_doc.doc_id().unwrap(),
                "ver": dummy_proposal_doc.doc_ver().unwrap()
            },
            "reply": {
                "id": comment_doc_id,
                "ver": comment_doc_ver
            }
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(dummy_template_doc).unwrap();
    provider.add_document(dummy_proposal_doc).unwrap();
    provider.add_document(comment_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid);
}

#[tokio::test]
async fn test_invalid_comment_doc() {
    let dummy_proposal_doc = dummy_proposal_doc();
    let dummy_template_doc = dummy_comment_template_doc();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": dummy_template_doc.doc_id().unwrap(),
              "ver": dummy_template_doc.doc_ver().unwrap()
            },
            // without ref
            "ref": serde_json::Value::Null
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(dummy_template_doc).unwrap();
    provider.add_document(dummy_proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
