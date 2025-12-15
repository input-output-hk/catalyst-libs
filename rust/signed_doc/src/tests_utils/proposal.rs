use ed25519_dalek::ed25519::signature::Signer;

use crate::{
    Builder, CatalystSignedDocument, ContentEncoding, ContentType, catalyst_id::role_index::RoleId,
    doc_types, providers::tests::TestCatalystProvider, tests_utils::create_dummy_key_pair,
    uuid::UuidV7,
};

/// # Errors
pub fn proposal_doc(
    template_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(RoleId::Proposer);
    provider.add_sk(kid.clone(), sk.clone());

    let template_ref = template_doc.doc_ref()?;
    let parameters_ref = parameters_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::PROPOSAL.clone(),
            "id": id,
            "ver": id,
            "template": [template_ref],
            "parameters": [parameters_ref]
        }))?
        .with_json_content(&serde_json::json!({}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
