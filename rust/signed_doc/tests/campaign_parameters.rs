//! Integration test for campaign parameters document validation part.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/campaign_parameters>

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::common::{
    brand_parameters_doc, brand_parameters_form_template_doc, campaign_parameters_doc,
    campaign_parameters_form_template_doc, create_dummy_key_pair,
};

mod common;

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        campaign_parameters_doc(&template, &parameters, provider)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
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
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
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
            .build()
    }
    => false
    ;
    "empty content"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
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
            .build()
    }
    => true
    ;
    "missing 'content-encoding' (optional)"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
                "parameters": {
                    "id": parameters.doc_id()?,
                    "ver": parameters.doc_ver()?,
                }
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'template'"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let parameters = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let template = campaign_parameters_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(None, v).unwrap())?;
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
                "template": {
                    "id": template.doc_id()?,
                    "ver": template.doc_ver()?,
                },
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'parameters'"
)]
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_campaign_parameters_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::CAMPAIGN_PARAMETERS.clone()
    );

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
