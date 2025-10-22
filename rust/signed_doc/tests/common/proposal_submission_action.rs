use catalyst_signed_doc::providers::tests::TestCatalystProvider;
use ed25519_dalek::ed25519::signature::Signer;

use super::*;

pub fn proposal_submission_action_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(RoleId::Proposer)
        .inspect(|(sk, kid)| provider.add_sk(kid.clone(), sk.clone()))?;
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::PROPOSAL_SUBMISSION_ACTION.clone(),
            "id": id,
            "ver": id,
            "ref": {
                "id": ref_doc.doc_id()?,
                "ver": ref_doc.doc_ver()?,
            },
            "parameters": {
                "id": parameters_doc.doc_id()?,
                "ver": parameters_doc.doc_ver()?,
            }
        }))?
        .with_json_content(&serde_json::json!({
            "action": "final"
        }))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
