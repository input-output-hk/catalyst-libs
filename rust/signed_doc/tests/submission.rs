//! Test for proposal submission action.

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

#[tokio::test]
async fn test_valid_submission_action() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .with_json_content(serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_valid_submission_action_old_type() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            // Using old (single uuid)
            "type": deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .with_json_content(serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_valid_submission_action_with_empty_provider() {
    let proposal_doc_id = UuidV7::new();
    let proposal_doc_ver = UuidV7::new();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();

    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .with_json_content(serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_submission_action() {
    let uuid_v7 = UuidV7::new();
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
    // missing `ref` field
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            // without specifying ref
            "ref": serde_json::Value::Null,
        }))
        .unwrap()
        .with_json_content(serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let provider = TestCatalystSignedDocumentProvider::default();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);

    // corrupted JSON
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .with_json_content(serde_json::Value::Null)
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);

    // empty content
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let uuid_v7 = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
        .unwrap()
        .build();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}
