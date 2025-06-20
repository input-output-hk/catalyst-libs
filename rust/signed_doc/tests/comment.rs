//! Test for Proposal Comment document.
//! Require fields: type, id, ver, ref, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment/>

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};
use catalyst_types::catalyst_id::role_index::RoleId;

mod common;

// Given a proposal comment document `doc`:
//
// - Parameters:
// The `parameters` field in `doc` points to a brand document.
// The parameter rule defines the link reference as `template`, This mean the document
// that `ref` field in `doc` points to (in this case = template_doc), must have the same
// `parameters` value as `doc`.
//
// - Reply:
// The `reply` field in `doc` points to another comment (`ref_doc`).
// The rule requires that the `ref` field in `ref_doc` must match the `ref` field in `doc`
#[tokio::test]
async fn test_valid_comment_doc() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(&doc_types::PROPOSAL.clone()).unwrap();
    let (brand_doc, brand_doc_id, brand_doc_ver) =
        common::create_dummy_doc(&doc_types::BRAND_PARAMETERS.clone()).unwrap();

    let ref_doc_id = UuidV7::new();
    let ref_doc_ver = UuidV7::new();

    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();

    // Create a ref document, which is a proposal comment type
    let (ref_doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": ref_doc_id.to_string(),
            "ver": ref_doc_ver.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }})),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

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
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let comment_doc_id = UuidV7::new();
    let comment_doc_ver = UuidV7::new();
    // Create a main comment doc, contain all fields mention in the document (except
    // revocations and section)
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": comment_doc_id.to_string(),
            "ver": comment_doc_ver.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "reply": {
                "id": ref_doc_id,
                "ver": ref_doc_ver
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        })),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();
    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &brand_doc).unwrap();
    provider.add_document(None, &proposal_doc).unwrap();
    provider.add_document(None, &ref_doc).unwrap();
    provider.add_document(None, &template_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

// The same as above but test with the old type
#[tokio::test]
async fn test_valid_comment_doc_old_type() {
    let (proposal_doc, proposal_doc_id, proposal_doc_ver) =
        common::create_dummy_doc(&deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.try_into().unwrap())
            .unwrap();
    let (brand_doc, brand_doc_id, brand_doc_ver) =
        common::create_dummy_doc(&doc_types::BRAND_PARAMETERS.clone()).unwrap();

    let ref_doc_id = UuidV7::new();
    let ref_doc_ver = UuidV7::new();

    let template_doc_id = UuidV7::new();
    let template_doc_ver = UuidV7::new();

    // Create a ref document, which is a proposal comment type
    let (ref_doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::COMMENT_DOCUMENT_UUID_TYPE,
            "id": ref_doc_id.to_string(),
            "ver": ref_doc_ver.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }})),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

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
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let comment_doc_id = UuidV7::new();
    let comment_doc_ver = UuidV7::new();
    // Create a main comment doc, Contain all fields mention in the document (except
    // revocations and section)
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::COMMENT_DOCUMENT_UUID_TYPE.clone(),
            "id": comment_doc_id.to_string(),
            "ver": comment_doc_ver.to_string(),
            "ref": {
                "id": proposal_doc_id,
                "ver": proposal_doc_ver
            },
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "reply": {
                "id": ref_doc_id,
                "ver": ref_doc_ver
            },
            "parameters": { "id": brand_doc_id, "ver": brand_doc_ver }
        })),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();
    let mut provider = TestCatalystSignedDocumentProvider::default();

    provider.add_document(None, &brand_doc).unwrap();
    provider.add_document(None, &proposal_doc).unwrap();
    provider.add_document(None, &ref_doc).unwrap();
    provider.add_document(None, &template_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_invalid_comment_doc() {
    let (proposal_doc, ..) =
        common::create_dummy_doc(&deprecated::PROPOSAL_DOCUMENT_UUID_TYPE.try_into().unwrap())
            .unwrap();
    let (template_doc, template_doc_id, template_doc_ver) =
        common::create_dummy_doc(&deprecated::COMMENT_TEMPLATE_UUID_TYPE.try_into().unwrap())
            .unwrap();

    let uuid_v7 = UuidV7::new();
    // Missing parameters field
    let (doc, ..) = common::create_dummy_signed_doc(
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "template": {
              "id": template_doc_id,
              "ver": template_doc_ver
            },
            "ref": serde_json::Value::Null
        }),
        serde_json::to_vec(&serde_json::Value::Null).unwrap(),
        RoleId::Role0,
    )
    .unwrap();

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(None, &template_doc).unwrap();
    provider.add_document(None, &proposal_doc).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();

    assert!(!is_valid);
}
