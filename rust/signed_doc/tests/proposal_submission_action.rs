//! Test for Proposal Submission Action.
//! Require fields: type, id, ver, ref, parameters
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/proposal_submission_action/>

use catalyst_signed_doc::{
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
        campaign_parameters_form_template_doc, category_parameters_doc,
        category_parameters_form_template_doc, create_dummy_key_pair, get_doc_kid_and_sk,
        proposal_doc, proposal_form_template_doc, proposal_submission_action_doc,
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
        proposal_submission_action_doc(&proposal, &parameters, provider)
    }
    => true
    ;
    "valid document with brand 'parameters'"
)]
#[test_case(
    |provider| {
       let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = campaign_parameters_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        proposal_submission_action_doc(&proposal, &parameters, provider)
    }
    => true
    ;
    "valid document with campaign 'parameters'"
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
        proposal_submission_action_doc(&proposal, &parameters, provider)
    }
    => true
    ;
    "valid document with category 'parameters'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &proposal, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Role0)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let proposal_ref = proposal.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "parameters": [parameters_ref]
            }))?
            .with_json_content(&serde_json::json!({
                "action": "final"
            }))?
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
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Proposer);
        provider.add_sk(kid.clone(), sk.clone());

        let proposal_ref = proposal.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "parameters": [parameters_ref]
            }))?
            .with_json_content(&serde_json::json!({
                "action": "final"
            }))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "singed by other kid"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &proposal, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let proposal_ref = proposal.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "parameters": [parameters_ref]
            }))?
            .empty_content()?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
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
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &proposal, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let proposal_ref = proposal.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "parameters": [parameters_ref]
            }))?
            .with_json_content(&serde_json::json!("null"))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "corrupted content"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &proposal, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let proposal_ref = proposal.doc_ref()?;
        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
                "parameters": [parameters_ref]
            }))?
            .with_json_content(&serde_json::json!({
                "action": "final"
            }))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
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
        let (sk, kid) = get_doc_kid_and_sk(provider, &proposal, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "parameters": [parameters_ref]
            }))?
            .with_json_content(&serde_json::json!({
                "action": "final"
            }))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing ref"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = get_doc_kid_and_sk(provider, &proposal, 0)
            .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
            .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

        let proposal_ref = proposal.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
                "id": id,
                "ver": id,
                "ref": [proposal_ref],
            }))?
            .with_json_content(&serde_json::json!({
                "action": "final"
            }))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())?
            .build()
    }
    => false
    ;
    "missing parameters"
)]
#[allow(clippy::unwrap_used)]
fn test_proposal_submission_action_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::PROPOSAL_SUBMISSION_ACTION.clone()
    );

    let is_valid = Validator::new().validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());
    is_valid
}
