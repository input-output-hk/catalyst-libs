//! Test for Proposal Comment document.
//! Require fields: type, id, ver, ref, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment/>

use std::sync::LazyLock;

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

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

#[allow(clippy::unwrap_used)]
static COMMENT_TEMPLATE_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
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

#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
        provider.add_pk(kid.clone(), pk);
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => false
    ;
    "wrong role"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .empty_content()?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => false
    ;
    "missing content"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "missing content-encoding (optional)"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": DUMMY_PROPOSAL_DOC.doc_id().unwrap(),
                    "ver": DUMMY_PROPOSAL_DOC.doc_ver().unwrap(),
                },
                "reply": {
                    "id": COMMENT_REF_DOC.doc_id().unwrap(),
                    "ver": COMMENT_REF_DOC.doc_ver().unwrap()
                },
                "parameters": {
                    "id": DUMMY_BRAND_DOC.doc_id().unwrap(),
                    "ver": DUMMY_BRAND_DOC.doc_ver().unwrap(),
                }
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => false
    ;
    "missing template"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => false
    ;
    "missing parameters"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => false
    ;
    "missing ref"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "missing reply (optional)"
)]
#[tokio::test]
async fn test_comment_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();

    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();
    provider.add_document(None, &DUMMY_PROPOSAL_DOC).unwrap();
    provider.add_document(None, &COMMENT_REF_DOC).unwrap();
    provider.add_document(None, &COMMENT_TEMPLATE_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
