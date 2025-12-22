//! Test for Proposal Comment document.
//! Require fields: type, id, ver, ref, template, parameters
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/proposal_comment>

use catalyst_signed_doc::{
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
        campaign_parameters_form_template_doc, category_parameters_doc,
        category_parameters_form_template_doc, create_dummy_key_pair, proposal_comment_doc,
        proposal_comment_form_template_doc, proposal_doc, proposal_form_template_doc,
    },
    validator::Validator,
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        proposal_comment_doc(&proposal, &template, &parameters, provider)
    }
    => true
    ;
    "valid document with brand 'parameters' and without 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = campaign_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        proposal_comment_doc(&proposal, &template, &parameters, provider)
    }
    => true
    ;
    "valid document with campaign 'parameters' and without 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = campaign_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = category_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = category_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        proposal_comment_doc(&proposal, &template, &parameters, provider)
    }
    => true
    ;
    "valid document with category 'parameters' and without 'reply'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let comment = proposal_comment_doc(&proposal, &template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let template_ref = template.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;
        let comment_ref = comment.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "template": [template_ref],
                "parameters": [parameters_ref],
                "reply": [comment_ref],
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Proposer);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let template_ref = template.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "template": [template_ref],
                "parameters": [parameters_ref],
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let template_ref = template.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "template": [template_ref],
                "parameters": [parameters_ref]
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let template_ref = template.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "template": [template_ref],
                "parameters": [parameters_ref]
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "parameters": [parameters_ref]
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let template_ref = template.doc_ref()?;

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "template": [template_ref],
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
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_comment_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let template_ref = template.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT.clone(),
                "id": id,
                "ver": id,
                "template": [template_ref],
                "parameters": [parameters_ref]
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
#[allow(clippy::unwrap_used)]
fn test_proposal_comment_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::PROPOSAL_COMMENT.clone()
    );

    let is_valid = Validator::new().validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());
    is_valid
}
