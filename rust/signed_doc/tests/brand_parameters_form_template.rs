//! Integration test for brand parameters form template document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/brand_parameters_form_template>

use catalyst_signed_doc::{
    builder::Builder,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_form_template_doc, create_dummy_admin_key_pair, create_dummy_key_pair,
        create_key_pair_and_publish,
    },
    validator::Validator,
    *,
};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

#[test_case(
    |p| {
        brand_parameters_form_template_doc(p)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |p| {
        let (sk, kid) = create_key_pair_and_publish(p, || create_dummy_key_pair(RoleId::Role0));
        let id = uuid::UuidV7::new();
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS_FORM_TEMPLATE.clone(),
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
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let id = uuid::UuidV7::new();
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "content-encoding": ContentEncoding::Brotli,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS_FORM_TEMPLATE.clone(),
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
        let (sk, kid) = create_key_pair_and_publish(p, create_dummy_admin_key_pair);
        let id = uuid::UuidV7::new();
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::SchemaJson,
                "id": id,
                "ver": id,
                "type": doc_types::BRAND_PARAMETERS_FORM_TEMPLATE.clone(),
            }))?
            .with_json_content(&serde_json::json!({}))?
            .add_signature(|m| sk.sign(&m).to_vec(), kid)?
            .build()
    }
    => true
    ;
    "missing 'content-encoding' (optional)"
)]
#[allow(clippy::unwrap_used)]
fn test_brand_parameters_form_template_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::BRAND_PARAMETERS_FORM_TEMPLATE.clone()
    );

    Validator::new().validate(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
