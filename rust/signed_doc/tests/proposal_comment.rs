//! Test for Proposal Comment document.
//! Require fields: type, id, ver, ref, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment/>

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::common::{
    create_dummy_key_pair,
    dummies::{
        BRAND_PARAMETERS_DOC, CAMPAIGN_PARAMETERS_DOC, CATEGORY_PARAMETERS_DOC,
        COMMENT_TEMPLATE_FOR_BRAND_DOC, COMMENT_TEMPLATE_FOR_CAMPAIGN_DOC,
        COMMENT_TEMPLATE_FOR_CATEGORY_DOC, PROPOSAL_COMMENT_FOR_BRAND_DOC,
        PROPOSAL_COMMENT_FOR_CAMPAIGN_DOC, PROPOSAL_COMMENT_FOR_CATEGORY_DOC,
        PROPOSAL_FOR_BRAND_DOC, PROPOSAL_FOR_CAMPAIGN_DOC, PROPOSAL_FOR_CATEGORY_DOC,
    },
};

mod common;

#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
                }
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "valid document with brand 'parameters'"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_CAMPAIGN_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_CAMPAIGN_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_CAMPAIGN_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_CAMPAIGN_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_CAMPAIGN_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_CAMPAIGN_DOC.doc_ver()?
                },
                "parameters": {
                    "id": CAMPAIGN_PARAMETERS_DOC.doc_id()?,
                    "ver": CAMPAIGN_PARAMETERS_DOC.doc_ver()?,
                }
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "valid document with campaign 'parameters'"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_CATEGORY_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_CATEGORY_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_CATEGORY_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_CATEGORY_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_CATEGORY_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_CATEGORY_DOC.doc_ver()?
                },
                "parameters": {
                    "id": CATEGORY_PARAMETERS_DOC.doc_id()?,
                    "ver": CATEGORY_PARAMETERS_DOC.doc_ver()?,
                }
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()?;
        Ok(doc)
    }
    => true
    ;
    "valid document with category 'parameters'"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Proposer)?;
        provider.add_pk(kid.clone(), pk);
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
        provider.add_pk(kid.clone(), pk);
        // Create a main comment doc, contain all fields mention in the document (except revocations)
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "reply": {
                    "id": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_COMMENT_FOR_BRAND_DOC.doc_ver()?
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
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
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0)?;
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
                    "id": PROPOSAL_FOR_BRAND_DOC.doc_id()?,
                    "ver": PROPOSAL_FOR_BRAND_DOC.doc_ver()?,
                },
                "template": {
                    "id": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_id()?,
                    "ver": COMMENT_TEMPLATE_FOR_BRAND_DOC.doc_ver()?,
                },
                "parameters": {
                    "id": BRAND_PARAMETERS_DOC.doc_id()?,
                    "ver": BRAND_PARAMETERS_DOC.doc_ver()?,
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
async fn test_proposal_comment_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();

    provider.add_document(None, &BRAND_PARAMETERS_DOC).unwrap();
    provider
        .add_document(None, &CAMPAIGN_PARAMETERS_DOC)
        .unwrap();
    provider
        .add_document(None, &CATEGORY_PARAMETERS_DOC)
        .unwrap();
    provider
        .add_document(None, &PROPOSAL_FOR_BRAND_DOC)
        .unwrap();
    provider
        .add_document(None, &PROPOSAL_FOR_CAMPAIGN_DOC)
        .unwrap();
    provider
        .add_document(None, &PROPOSAL_FOR_CATEGORY_DOC)
        .unwrap();
    provider
        .add_document(None, &PROPOSAL_COMMENT_FOR_BRAND_DOC)
        .unwrap();
    provider
        .add_document(None, &PROPOSAL_COMMENT_FOR_CAMPAIGN_DOC)
        .unwrap();
    provider
        .add_document(None, &PROPOSAL_COMMENT_FOR_CATEGORY_DOC)
        .unwrap();
    provider
        .add_document(None, &COMMENT_TEMPLATE_FOR_BRAND_DOC)
        .unwrap();
    provider
        .add_document(None, &COMMENT_TEMPLATE_FOR_CAMPAIGN_DOC)
        .unwrap();
    provider
        .add_document(None, &COMMENT_TEMPLATE_FOR_CATEGORY_DOC)
        .unwrap();

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
