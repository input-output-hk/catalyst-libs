//! Integration test for proposal form template document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/proposal_form_template/>

use catalyst_signed_doc::{
    builder::Builder,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
        campaign_parameters_form_template_doc, category_parameters_doc,
        category_parameters_form_template_doc, create_dummy_admin_key_pair, create_dummy_key_pair,
        proposal_form_template_doc,
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
        proposal_form_template_doc(&parameters, provider)
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
        proposal_form_template_doc(&parameters, provider)
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
        proposal_form_template_doc(&parameters, provider)
    }
    => true
    ;
    "valid document with category 'parameters'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
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
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
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
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());

        let parameters_ref = parameters.doc_ref()?;

        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
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
        let id = uuid::UuidV7::new();
        let (sk, kid) = create_dummy_admin_key_pair();
        provider.add_sk(kid.clone(), sk.clone());
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
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
#[allow(clippy::unwrap_used)]
fn test_proposal_form_template_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::PROPOSAL_FORM_TEMPLATE.clone()
    );

    Validator::new().validate(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
