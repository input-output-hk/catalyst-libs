//! Test for Proposal Submission Action.
//! Require fields: type, id, ver, ref, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_submission_action/>

use std::sync::LazyLock;

use catalyst_signed_doc::{
    doc_types::deprecated,
    providers::tests::{TestCatalystSignedDocumentProvider, TestVerifyingKeyProvider},
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

#[allow(clippy::unwrap_used)]
static DUMMY_PROPOSAL_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::PROPOSAL.clone(),
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .empty_content()
        .unwrap()
        .build()
        .unwrap()
});

#[allow(clippy::unwrap_used)]
static DUMMY_BRAND_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::BRAND_PARAMETERS.clone(),
        }))
        .unwrap()
        .empty_content()
        .unwrap()
        .build()
        .unwrap()
});

// Given a proposal comment document `doc`:
//
// - Parameters:
// The `parameters` field in `doc` points to a brand document.
// The parameter rule defines the link reference as `ref`, This mean the document that
// `ref` field in `doc` points to (in this case = `proposal_doc`), must have the same
// `parameters` value as `doc`.
#[tokio::test]
async fn test_valid_submission_action() {
    let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
    let mut key_provider = TestVerifyingKeyProvider::default();
    key_provider.add_pk(kid.clone(), pk);

    // Create a main proposal submission doc, contain all fields mention in the document
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid)
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());

    let is_valid = validator::validate_signatures(&doc, &key_provider)
        .await
        .unwrap();
    assert!(is_valid);
}

#[tokio::test]
async fn test_invalid_submission_action_wrong_role() {
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    // Create a main proposal submission doc, contain all fields mention in the document
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid)
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_valid_submission_action_old_type() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_invalid_submission_action_corrupted_json() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::Value::Null)
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_submission_action_missing_ref() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            // "ref": {
            //     "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
            //     "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            // },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_submission_action_missing_parameters() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            // "parameters": {
            //     "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
            //     "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            // }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}
