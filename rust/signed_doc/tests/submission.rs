//! Test for proposal submission action.

use catalyst_signed_doc::{
    providers::tests::TestCatalystSignedDocumentProvider,
    validator::tests::{TEST_FUTURE_THRESHOLD, TEST_PAST_THRESHOLD},
    *,
};

mod common;

#[tokio::test]
async fn test_valid_submission_action() {
    let (proposal_doc, proposal_doc_id) =
        common::create_dummy_doc(doc_types::PROPOSAL_DOCUMENT_UUID_TYPE).unwrap();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": {
            "id": proposal_doc_id
        },
    })))
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(proposal_doc).unwrap();

    let is_valid = validator::validate(
        &doc,
        Some(TEST_FUTURE_THRESHOLD),
        Some(TEST_PAST_THRESHOLD),
        &provider,
    )
    .await
    .unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_submission_action_with_empty_provider() {
    let proposal_doc_id = UuidV7::new();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        "ref": {
            "id": proposal_doc_id
        },
    })))
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(
        &doc,
        Some(TEST_FUTURE_THRESHOLD),
        Some(TEST_PAST_THRESHOLD),
        &provider,
    )
    .await
    .unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_submission_action() {
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(Some(serde_json::json!({
        "content-type": ContentType::Json.to_string(),
        "content-encoding": ContentEncoding::Brotli.to_string(),
        "type": doc_types::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        "id": uuid_v7.to_string(),
        "ver": uuid_v7.to_string(),
        // without specifying ref
        "ref": serde_json::Value::Null,
    })))
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(
        &doc,
        Some(TEST_FUTURE_THRESHOLD),
        Some(TEST_PAST_THRESHOLD),
        &provider,
    )
    .await
    .unwrap();

    assert!(!is_valid);
}
