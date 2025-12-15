use ed25519_dalek::ed25519::signature::Signer;

use super::*;
use crate::providers::tests::TestCatalystProvider;

pub fn rep_nomination_doc(
    template_doc: &CatalystSignedDocument,
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(RoleId::DelegatedRepresentative);
    provider.add_sk(kid.clone(), sk.clone());

    let template_ref = template_doc.doc_ref()?;
    let ref_ref = ref_doc.doc_ref()?;
    let parameters_ref = parameters_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::REP_NOMINATION.clone(),
            "id": id,
            "ver": id,
            "template": [template_ref],
            "ref": [ref_ref],
            "parameters": [parameters_ref],
        }))?
        .with_json_content(&serde_json::json!({}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
