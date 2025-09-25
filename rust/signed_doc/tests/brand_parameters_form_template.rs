//! Integration test for brand parameters form template document validation part.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/brand_parameters_form_template>

use catalyst_signed_doc::{providers::tests::TestCatalystProvider, *};
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;
use test_case::test_case;

use crate::common::{brand_parameters_form_template_doc, create_dummy_key_pair};

mod common;

#[test_case(
    |provider| {
        brand_parameters_form_template_doc(provider)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |provider| {
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
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
    |provider| {
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
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
    |provider| {
        let id = UuidV7::new();
        let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
            .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
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
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn test_brand_parameters_form_template_doc(
    doc_gen: impl FnOnce(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();

    let doc = doc_gen(&mut provider).unwrap();
    assert_eq!(
        *doc.doc_type().unwrap(),
        doc_types::BRAND_PARAMETERS_FORM_TEMPLATE.clone()
    );

    let is_valid = validator::validate(&doc, &provider).await.unwrap();
    assert_eq!(is_valid, !doc.problem_report().is_problematic());
    println!("{:?}", doc.problem_report());
    is_valid
}
