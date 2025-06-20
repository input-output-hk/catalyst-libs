//! Test for Proposal Submission Action.
//! Require fields: type, id, ver, ref, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_submission_action/>

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};
use catalyst_types::catalyst_id::role_index::RoleId;

mod common;

// Given a proposal comment document `doc`:
//
// - Parameters:
// The `parameters` field in `doc` points to a brand document.
// The parameter rule defines the link reference as `ref`, This mean the document that
// `ref` field in `doc` points to (in this case = `proposal_doc`), must have the same
// `parameters` value as `doc`.
#[tokio::test]
async fn test_valid_submission_action() {
    let (brand_doc, brand_doc_id, brand_doc_ver) =
        common::create_dummy_doc(&doc_types::BRAND_PARAMETERS.clone()).unwrap();

    let proposal_doc_id = UuidV7::new();
    let proposal_doc_ver = UuidV7::new();
    let (proposal_doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": proposal_doc_id.to_string(),
            "ver": proposal_doc_ver.to_string(),
            "ref": {
                "id": UuidV7::new(),
                "ver": UuidV7::new()
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let doc_id = UuidV7::new();
    let doc_ver = UuidV7::new();
    // Create a main proposal submission doc, contain all fields mention in the document
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": doc_id.to_string(),
            "ver": doc_ver.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &proposal_doc).unwrap();
    provider.add_document(None, &brand_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_valid_submission_action_old_type() {
    let (brand_doc, brand_doc_id, brand_doc_ver) =
        common::create_dummy_doc(&doc_types::BRAND_PARAMETERS.clone()).unwrap();

    let proposal_doc_id = UuidV7::new();
    let proposal_doc_ver = UuidV7::new();
    let (proposal_doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.clone(),
            "id": proposal_doc_id.to_string(),
            "ver": proposal_doc_ver.to_string(),
            "ref": {
                "id": UuidV7::new(),
                "ver": UuidV7::new()
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let doc_id = UuidV7::new();
    let doc_ver = UuidV7::new();
    // Create a main proposal submission doc, contain all fields mention in the document
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE.clone(),
            "id": doc_id.to_string(),
            "ver": doc_ver.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        }),
        serde_json::to_vec(&serde_json::json!({
            "action": "final"
        }))
        .unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &proposal_doc).unwrap();
    provider.add_document(None, &brand_doc).unwrap();

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
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
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
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
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
        common::create_dummy_doc(&deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.try_into().unwrap())
            .unwrap();
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
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

    provider.add_document(None, &proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);

    // empty content
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(&deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.try_into().unwrap())
            .unwrap();
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
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

    provider.add_document(None, &proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}
