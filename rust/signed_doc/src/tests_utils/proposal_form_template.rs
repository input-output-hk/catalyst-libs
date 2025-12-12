use ed25519_dalek::ed25519::signature::Signer;

use super::*;
use crate::providers::tests::TestCatalystProvider;

pub fn proposal_form_template_doc(
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(None);
    provider.add_sk(kid.clone(), sk.clone());

    let parameters_ref = parameters_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::SchemaJson,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
            "id": id,
            "ver": id,
            "parameters": [parameters_ref],
        }))?
        .with_json_content(&serde_json::json!({}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
