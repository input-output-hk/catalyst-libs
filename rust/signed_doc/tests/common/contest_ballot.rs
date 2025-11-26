use catalyst_signed_doc::providers::tests::TestCatalystProvider;
use ed25519_dalek::ed25519::signature::Signer;

use super::*;

/// Creates a contest ballot document.
pub fn contest_ballot_doc(
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(None);
    provider.add_sk(kid.clone(), sk.clone());

    let parameters_ref = parameters_doc.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Cbor,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_BALLOT.clone(),
            "id": id,
            "ver": id,
            "parameters": [parameters_ref],
        }))?
        .with_cbor_content(vec![1])?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
