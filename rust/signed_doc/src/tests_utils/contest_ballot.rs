use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::ed25519::signature::Signer;

use crate::{
    Builder, CatalystSignedDocument, ContentEncoding, ContentType, doc_types,
    providers::tests::TestCatalystProvider, tests_utils::create_dummy_key_pair, uuid::UuidV7,
};

#[allow(clippy::missing_errors_doc)]
pub fn contest_ballot_doc(
    ref_doc: &CatalystSignedDocument,
    parameters_doc: &CatalystSignedDocument,
    provider: &mut TestCatalystProvider,
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    let (sk, kid) = create_dummy_key_pair(RoleId::Role0);
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
        .with_raw_cbor_content(&[160])?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
