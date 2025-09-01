//! Integration test for proposal document validation part.
//! Require fields: type, id, ver, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal/#front-end>

use std::sync::LazyLock;

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::common::create_dummy_key_pair;

mod common;

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

#[allow(clippy::unwrap_used)]
static PROPOSAL_TEMPLATE_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "parameters": {
                    "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                    "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
                },
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }))
        .unwrap()
        .build()
        .unwrap()
});

// Given a proposal document `doc`:
//
// - Parameters:
// The `parameters` field in `doc` points to a brand document.
// The parameter rule defines the link reference as `template`, This mean the document
// that `ref` field in `doc` points to (in this case = `template_doc`), must have the same
// `parameters` value as `doc`.
#[tokio::test]
async fn test_valid_proposal_doc() {
    let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
    let mut provider = TestCatalystProvider::default();
    provider.add_pk(kid.clone(), pk);

    // Create a main proposal doc, contain all fields mention in the document (except
    // collaborations and revocations)
    let id = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": id,
            "ver": id,
            "template": {
                "id": PROPOSAL_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": PROPOSAL_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({}))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid)
        .unwrap()
        .build()
        .unwrap();

    provider.add_document(None, &PROPOSAL_TEMPLATE_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid);
    assert!(is_valid);
    assert!(!doc.problem_report().is_problematic());
}

#[tokio::test]
async fn test_invalid_proposal_doc_wrong_role() {
    let (sk, _pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();

    // Create a main proposal doc, contain all fields mention in the document (except
    // collaborations and revocations)
    let id = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": id,
            "ver": id,
            "template": {
                "id": PROPOSAL_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": PROPOSAL_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({}))
        .unwrap()
        .add_signature(|m| sk.sign(&m).to_vec(), kid)
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystProvider::default();

    provider.add_document(None, &PROPOSAL_TEMPLATE_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc_missing_template() {
    let id = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": id,
            "ver": id,
            // "template": {
            //     "id": PROPOSAL_TEMPLATE_DOC.doc_id().unwrap(),
            //     "ver": PROPOSAL_TEMPLATE_DOC.doc_ver().unwrap(),
            // },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystProvider::default();

    provider.add_document(None, &PROPOSAL_TEMPLATE_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc_missing_parameters() {
    let id = UuidV7::new();
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": id,
            "ver": id,
            "template": {
                "id": PROPOSAL_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": PROPOSAL_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            // "parameters": {
            //     "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
            //     "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            // }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap();

    let mut provider = TestCatalystProvider::default();

    provider.add_document(None, &PROPOSAL_TEMPLATE_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}
