use ed25519_dalek::ed25519::signature::Signer;

use super::*;
use crate::providers::tests::TestCatalystProvider;

/// Creates a contest ballot document.
pub fn contest_ballot_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(None);
    provider.add_sk(kid.clone(), sk.clone());

    let parameters_ref = parameters_doc.doc_ref()?;
    let ref_ref = ref_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Cbor,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_BALLOT.clone(),
            "id": id,
            "ver": id,
            "ref": [ref_ref],
            "parameters": [parameters_ref],
        }))?
        .with_cbor_content(1)?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
