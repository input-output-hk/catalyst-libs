//! Test for proposal submission action.

use catalyst_signed_doc::{providers::tests::TestCatalystSignedDocumentProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;

mod common;

#[tokio::test]
async fn test_valid_submission_action() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_valid_submission_action_with_empty_provider() {
    let proposal_doc_id = UuidV7::new();
    let proposal_doc_ver = UuidV7::new();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_submission_action() {
    let uuid_v7 = UuidV7::new();
    // missing `ref` field
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            // without specifying ref
            "ref": serde_json::Value::Null,
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);

    // corrupted JSON
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);

    // empty content
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
        }),
        vec![],
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();
    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}
