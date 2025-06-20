//! Integration test for proposal document validation part.

use catalyst_signed_doc::{providers::tests::TestCatalystSignedDocumentProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

fn dummy_proposal_template() -> CatalystSignedDocument {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::deprecated::PROPOSAL_TEMPLATE_UUID_TYPE,
        }))
        .unwrap()
        .with_json_content(serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_valid_proposal_doc() {
    let dummy_template_doc = dummy_proposal_template();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": dummy_template_doc.doc_id().unwrap(),
              "ver": dummy_template_doc.doc_ver().unwrap()
            },
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

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_proposal_doc_old_type() {
    let dummy_template_doc = dummy_proposal_template();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            // Using old (single uuid)
            "type": doc_types::deprecated::PROPOSAL_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": dummy_template_doc.doc_id().unwrap(),
              "ver": dummy_template_doc.doc_ver().unwrap()
            },
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

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_proposal_doc_with_empty_provider() {
    // dummy template doc to dummy provider
    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc() {
    let uuid_v7 = UuidV7::new();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            // without specifying template id
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build()
        .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
