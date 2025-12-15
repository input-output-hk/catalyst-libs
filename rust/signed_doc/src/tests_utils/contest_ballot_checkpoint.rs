use ed25519_dalek::ed25519::signature::Signer;

use super::*;
use crate::providers::tests::TestCatalystProvider;

/// Creates a contest ballot checkpoint document.
pub fn contest_ballot_checkpoint_doc(
    linked_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_admin_key_pair();
    provider.add_sk(kid.clone(), sk.clone());

    let linked_ref = linked_doc.doc_ref()?;
    let parameters_ref = parameters_doc.doc_ref()?;
    let chain = Chain::new(0, None);

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Cbor,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
            "id": id,
            "ver": id,
            "ref": [linked_ref],
            "parameters": [parameters_ref],
            "chain": chain,
        }))?
        .with_cbor_content(1)?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
