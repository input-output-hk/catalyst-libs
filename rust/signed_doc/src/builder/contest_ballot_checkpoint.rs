use ed25519_dalek::{SigningKey, ed25519::signature::Signer};

use crate::{
    CatalystSignedDocument, ContentEncoding, ContentType, builder::Builder,
    catalyst_id::CatalystId, doc_types, metadata::Chain, uuid::UuidV7,
};

pub fn contest_ballot_checkpoint_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    sk: &SigningKey,
    kid: CatalystId,
    id: Option<UuidV7>,
) -> anyhow::Result<CatalystSignedDocument> {
    let (id, ver) = id.map_or_else(
        || {
            let id = UuidV7::new();
            (id, id)
        },
        |v| (v, UuidV7::new()),
    );
    let linked_ref = linked.doc_ref()?;
    let parameters_ref = parameters.doc_ref()?;
    let chain = Chain::new(0, None);

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Cbor,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_BALLOT_CHECKPOINT.clone(),
            "id": id,
            "ver": ver,
            "ref": [linked_ref],
            "parameters": [parameters_ref],
            "chain": chain,
        }))?
        .with_cbor_content(1)?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
