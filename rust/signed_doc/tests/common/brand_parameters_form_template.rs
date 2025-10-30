use catalyst_signed_doc::providers::tests::TestCatalystProvider;
use ed25519_dalek::ed25519::signature::Signer;

use super::*;

pub fn brand_parameters_form_template_doc(
    provider: &mut TestCatalystProvider
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(None);
    provider.add_sk(kid.clone(), sk.clone());
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
