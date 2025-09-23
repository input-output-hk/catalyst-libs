use catalyst_signed_doc::providers::tests::TestCatalystProvider;
use ed25519_dalek::ed25519::signature::Signer;

use super::*;

/// Creates a Proposal Comment doc, without reply metadata field
pub fn proposal_comment_doc(
    ref_doc: &CatalystSignedDocument,
    template_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0)
        .inspect(|(_, pk, kid)| provider.add_pk(kid.clone(), *pk))?;
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT.clone(),
            "id": id,
            "ver": id,
            "ref": {
                "id": ref_doc.doc_id()?,
                "ver": ref_doc.doc_ver()?,
            },
            "template": {
                "id": template_doc.doc_id()?,
                "ver": template_doc.doc_ver()?,
            },
            "parameters": {
                "id": parameters_doc.doc_id()?,
                "ver": parameters_doc.doc_ver()?,
            }
        }))?
        .with_json_content(&serde_json::json!({}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
