//! Integration test for campaign parameters form template document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/campaign_parameters_form_template>

use catalyst_signed_doc::{
    providers::tests::TestCatalystProvider, tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc,
        campaign_parameters_form_template_doc, create_dummy_admin_key_pair, create_dummy_key_pair,
    }, validator::Validator, *
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        campaign_parameters_form_template_doc(&parameters, provider)
    }
    => true
    ;
    "valid document"
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
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS_FORM_TEMPLATE.clone(),
                "parameters": [parameters_ref]
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
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS_FORM_TEMPLATE.clone(),
                "parameters": [parameters_ref]
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
                "content-type": ContentType::SchemaJson,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS_FORM_TEMPLATE.clone(),
                "parameters": [parameters_ref]
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
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS_FORM_TEMPLATE.clone(),
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'parameters'"
)]
#[allow(clippy::unwrap_used)]
fn test_campaign_parameters_form_template_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::CAMPAIGN_PARAMETERS_FORM_TEMPLATE.clone()
    );

    let is_valid = Validator::new().validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());
    is_valid
}
