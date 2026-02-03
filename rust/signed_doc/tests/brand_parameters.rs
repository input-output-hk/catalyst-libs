//! Integration test for brand parameters document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/brand_parameters>

use catalyst_signed_doc::{
    builder::Builder,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, build_verify_and_publish,
        create_dummy_admin_key_pair, create_dummy_key_pair, create_key_pair_and_publish,
    },
    validator::Validator,
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

#[test_case(
    |p| {
        let template = build_verify_and_publish(p, brand_parameters_form_template_doc)?;
        brand_parameters_doc(&template, p)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |p| {
        let template = build_verify_and_publish(p, brand_parameters_form_template_doc)?;

        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let id = uuid::UuidV7::new();
        let template_ref = template.doc_ref()?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS.clone(),
                "template": [template_ref],
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
    |p| {
        let template = build_verify_and_publish(p, brand_parameters_form_template_doc)?;

        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let id = uuid::UuidV7::new();
        let template_ref = template.doc_ref()?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS.clone(),
                "template": [template_ref],
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
    |p| {
        let template = build_verify_and_publish(p, brand_parameters_form_template_doc)?;

        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let id = uuid::UuidV7::new();
        let template_ref = template.doc_ref()?;
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS.clone(),
                "template": [template_ref],
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
                "content-type": ContentType::Json,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS.clone(),
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => false
    ;
    "missing 'template'"
)]
#[allow(clippy::unwrap_used)]
fn test_brand_parameters_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::BRAND_PARAMETERS.clone()
    );

    Validator::new().validate(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
