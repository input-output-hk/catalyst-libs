//! Integration test for proposal document validation part.
//! Require fields: type, id, ver, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal/#front-end>

use std::sync::LazyLock;

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

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

#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
        provider.add_pk(kid.clone(), pk);
        // Create a main proposal doc, contain all fields mention in the document (except
        // 'collaborators' and 'revocations')
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0).unwrap();
        provider.add_pk(kid.clone(), pk);
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
        provider.add_pk(kid.clone(), pk);
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
            }))?
            .empty_content()?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?    
            .build()?;
        Ok(doc)
    }
    => false
    ;
    "empty content"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
        provider.add_pk(kid.clone(), pk);
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
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
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?    
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "missing 'content-encoding' (optional)"
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
                "type": doc_types::PROPOSAL.clone(),
                "id": id,
                "ver": id,
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer).unwrap();
        provider.add_pk(kid.clone(), pk);
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
#[tokio::test]
async fn test_proposal_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();


    let doc = doc_gen(&mut provider).unwrap();

    provider.add_document(None, &PROPOSAL_TEMPLATE_DOC).unwrap();
    provider.add_document(None, &DUMMY_BRAND_DOC).unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
