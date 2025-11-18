//! Integration test for proposal comment form template document validation part.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment_form_template>

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::common::{
    brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
    campaign_parameters_form_template_doc, category_parameters_doc,
    category_parameters_form_template_doc, create_dummy_key_pair,
    proposal_comment_form_template_doc,
};

mod common;

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        proposal_comment_form_template_doc(&parameters, provider)
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
        proposal_comment_form_template_doc(&parameters, provider)
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
        proposal_comment_form_template_doc(&parameters, provider)
    }
    => true
    ;
    "valid document with category 'parameters'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(Some(RoleId::Role0));
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
                "id": id,
                "ver": id,
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
        let id = UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(None);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
                "id": id,
                "ver": id,
                "parameters": [parameters_ref],
            }))?
            .empty_content()?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "empty content"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(None);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
                "id": id,
                "ver": id,
                "parameters": [parameters_ref],
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => true
    ;
    "missing 'content-encoding' (optional)"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(None);
        provider.add_sk(kid.clone(), sk.clone());
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
                "id": id,
                "ver": id,
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing parameters"
)]
#[tokio::test]
#[ignore = "Broken Test Case, TODO: Fix it."]
#[allow(clippy::unwrap_used)]
async fn test_proposal_comment_form_template_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone()
    );

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
