use ed25519_dalek::ed25519::signature::Signer;

use crate::{
    Builder, CatalystSignedDocument, ContentEncoding, ContentType, catalyst_id::role_index::RoleId,
    doc_types, providers::tests::TestCatalystProvider, tests_utils::get_doc_kid_and_sk,
    uuid::UuidV7,
};

/// # Errors
pub fn proposal_submission_action_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = get_doc_kid_and_sk(provider, ref_doc, 0)
        .map(|(sk, kid)| (sk, kid.with_role(RoleId::Proposer)))
        .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;

    let ref_doc_ref = ref_doc.doc_ref()?;
    let parameters_doc_ref = parameters_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": id,
            "ver": id,
            "ref": [ref_doc_ref],
            "parameters": [parameters_doc_ref]
        }))?
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())?
        .build()
}
