//! Integration test for proposal document validation part.
//! Require fields: type, id, ver, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal/#front-end>

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};
use catalyst_types::catalyst_id::role_index::RoleId;

mod common;

// Given a proposal document `doc`:
//
// - Parameters:
// The `parameters` field in `doc` points to a brand document.
// The parameter rule defines the link reference as `template`, This mean the document
// that `ref` field in `doc` points to (in this case = `template_doc`), must have the same
// `parameters` value as `doc`.
#[tokio::test]
async fn test_valid_proposal_doc() {
    let (brand_doc, brand_doc_id, brand_doc_ver) =
        common::create_dummy_doc(&doc_types::BRAND_PARAMETERS.clone()).unwrap();

    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();
    // Create a template document
    let (template_doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_TEMPLATE.clone(),
            "id": template_doc_id.to_string(),
            "ver": template_doc_ver.to_string(),
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        })),
        serde_json::to_vec(&serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }))
        .unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let proposal_doc_id = UuidV7::new();
    let proposal_doc_ver = UuidV7::new();
    // Create a main proposal doc, contain all fields mention in the document (except
    // collaborations and revocations)
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": proposal_doc_id.to_string(),
            "ver": proposal_doc_ver.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "parameters": {
                "id": brand_doc_id,
                "ver": brand_doc_ver
            }
        }),
        // Validate against the ref template
        serde_json::to_vec(&serde_json::json!({})).unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &template_doc).unwrap();
    provider.add_document(None, &brand_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_proposal_doc_old_type() {
    let (brand_doc, brand_doc_id, brand_doc_ver) =
        common::create_dummy_doc(&doc_types::BRAND_PARAMETERS.clone()).unwrap();

    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();
    // Create a template document
    let (template_doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_TEMPLATE.clone(),
            "id": template_doc_id.to_string(),
            "ver": template_doc_ver.to_string(),
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        })),
        serde_json::to_vec(&serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }))
        .unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let proposal_doc_id = UuidV7::new();
    let proposal_doc_ver = UuidV7::new();
    // Create a main proposal doc, contain all fields mention in the document (except
    // collaborations and revocations)
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.clone(),
            "id": proposal_doc_id.to_string(),
            "ver": proposal_doc_ver.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "parameters": {
                "id": brand_doc_id,
                "ver": brand_doc_ver
            }
        }),
        serde_json::to_vec(&serde_json::json!({})).unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &template_doc).unwrap();
    provider.add_document(None, &brand_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(is_valid);
}

#[tokio::test]
async fn test_valid_proposal_doc_with_empty_provider() {
    // dummy template doc to dummy provider
    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();

    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_proposal_doc() {
    let uuid_v7 = UuidV7::new();
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            // without specifying template id
            "template": serde_json::Value::Null,
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Proposer,
    )
    .unwrap();

    let provider = TestCatalystSignedDocumentProvider::default();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
