use catalyst_signed_doc::providers::tests::TestCatalystProvider;
use ed25519_dalek::ed25519::signature::Signer;

use super::*;

pub fn proposal_comment_form_template_doc(
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, _, kid) = create_dummy_key_pair(RoleId::BrandAdmin)
        .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::SchemaJson,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
            "id": id,
            "ver": id,
            "parameters": {
                    "id": parameters_doc.doc_id()?,
                    "ver": parameters_doc.doc_ver()?,
                }
        }))?
        .with_json_content(&serde_json::json!({}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
