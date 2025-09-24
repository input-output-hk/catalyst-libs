//! Test for Proposal Comment document.
//! Require fields: type, id, ver, ref, template, parameters
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment/>

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::common::{
    brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
    campaign_parameters_form_template_doc, category_parameters_doc,
    category_parameters_form_template_doc, create_dummy_key_pair, proposal_comment_doc,
    proposal_comment_form_template_doc, proposal_doc, proposal_form_template_doc,
};

mod common;

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        proposal_comment_doc(&proposal, &template, &parameters, provider)
    }
    => true
    ;
    "valid document with brand 'parameters' and without 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = campaign_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        proposal_comment_doc(&proposal, &template, &parameters, provider)
    }
    => true
    ;
    "valid document with campaign 'parameters' and without 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = campaign_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = category_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = category_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        proposal_comment_doc(&proposal, &template, &parameters, provider)
    }
    => true
    ;
    "valid document with category 'parameters' and without 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let comment = proposal_comment_doc(&proposal, &template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": proposal.doc_id()?,
                    "ver": proposal.doc_ver()?,
                },
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
                },
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
                },
                "reply": {
                    "id": comment.doc_id()?,
                    "ver": comment.doc_ver()?
                },
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => true
    ;
    "valid document with brand 'parameters' and with 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Proposer)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": proposal.doc_id()?,
                    "ver": proposal.doc_ver()?,
                },
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
                },
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
                },
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "wrong role"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": proposal.doc_id()?,
                    "ver": proposal.doc_ver()?,
                },
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
                },
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": proposal.doc_id()?,
                    "ver": proposal.doc_ver()?,
                },
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
                },
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": proposal.doc_id()?,
                    "ver": proposal.doc_ver()?,
                },
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": {
                    "id": proposal.doc_id()?,
                    "ver": proposal.doc_ver()?,
                },
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
                },
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
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
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_proposal_comment_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::PROPOSAL_COMMENT.clone()
    );

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
