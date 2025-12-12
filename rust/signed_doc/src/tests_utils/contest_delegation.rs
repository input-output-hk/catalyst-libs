use ed25519_dalek::ed25519::signature::Signer;

use super::*;
use crate::providers::tests::TestCatalystProvider;

/// Creates a contest delegation document.
pub fn contest_delegation_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(Some(RoleId::Role0));
    provider.add_sk(kid.clone(), sk.clone());

    let parameters_ref = parameters_doc.doc_ref()?;
    let ref_ref = ref_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_DELEGATION.clone(),
            "id": id,
            "ver": id,
            "ref": [ref_ref],
            "parameters": [parameters_ref],
        }))?
        .with_json_content(&serde_json::json!({"weights" : []}))?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
