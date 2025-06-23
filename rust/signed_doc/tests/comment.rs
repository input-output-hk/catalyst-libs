//! Test for Proposal Comment document.
//! Require fields: type, id, ver, ref, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment/>

use std::sync::LazyLock;

use catalyst_signed_doc::{
    doc_types::deprecated, providers::tests::TestCatalystSignedDocumentProvider, *,
};

#[allow(clippy::unwrap_used)]
static DUMMY_PROPOSAL_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::PROPOSAL.clone(),
        }))
        .unwrap()
        .empty_content()
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
        .build()
        .unwrap()
});

#[allow(clippy::unwrap_used)]
static COMMENT_TEMPLATE_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT_TEMPLATE.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
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

#[allow(clippy::unwrap_used)]
static COMMENT_REF_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "template": {
                "id": COMMENT_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": COMMENT_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "parameters": {
                "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
            }
        }))
        .unwrap()
        .with_json_content(&serde_json::json!({}))
        .unwrap()
        .build()
        .unwrap()
});

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
    // Create a main comment doc, contain all fields mention in the document (except
    // revocations and section)
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "template": {
                "id": COMMENT_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": COMMENT_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "reply": {
                "id": COMMENT_REF_DOC.doc_id().unwrap(),
                "ver": COMMENT_REF_DOC.doc_ver().unwrap()
            },
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

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();
    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &COMMENT_REF_DOC).unwrap();
    provider.add_document(None, &COMMENT_TEMPLATE_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

// The same as above but test with the old type
#[tokio::test]
async fn test_valid_comment_doc_old_type() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": deprecated::COMMENT_DOCUMENT_UUID_TYPE,
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "template": {
                "id": COMMENT_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": COMMENT_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "reply": {
                "id": COMMENT_REF_DOC.doc_id().unwrap(),
                "ver": COMMENT_REF_DOC.doc_ver().unwrap()
            },
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

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();
    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &COMMENT_REF_DOC).unwrap();
    provider.add_document(None, &COMMENT_TEMPLATE_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(is_valid, "{:?}", doc.problem_report());
}

#[tokio::test]
async fn test_invalid_comment_doc_missing_parameters() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            "template": {
                "id": COMMENT_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": COMMENT_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "reply": {
                "id": COMMENT_REF_DOC.doc_id().unwrap(),
                "ver": COMMENT_REF_DOC.doc_ver().unwrap()
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

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();
    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &COMMENT_REF_DOC).unwrap();
    provider.add_document(None, &COMMENT_TEMPLATE_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_comment_doc_missing_template() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "ref": {
                "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            },
            // "template": {
            //     "id": COMMENT_TEMPLATE_DOC.doc_id().unwrap(),
            //     "ver": COMMENT_TEMPLATE_DOC.doc_ver().unwrap(),
            // },
            "reply": {
                "id": COMMENT_REF_DOC.doc_id().unwrap(),
                "ver": COMMENT_REF_DOC.doc_ver().unwrap()
            },
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

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();
    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &COMMENT_REF_DOC).unwrap();
    provider.add_document(None, &COMMENT_TEMPLATE_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_invalid_comment_doc_missing_ref() {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            // "ref": {
            //     "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
            //     "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
            // },
            "template": {
                "id": COMMENT_TEMPLATE_DOC.doc_id().unwrap(),
                "ver": COMMENT_TEMPLATE_DOC.doc_ver().unwrap(),
            },
            "reply": {
                "id": COMMENT_REF_DOC.doc_id().unwrap(),
                "ver": COMMENT_REF_DOC.doc_ver().unwrap()
            },
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

    let mut provider = TestCatalystSignedDocumentProvider::default();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();
    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &COMMENT_REF_DOC).unwrap();
    provider.add_document(None, &COMMENT_TEMPLATE_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert!(!is_valid);
}
