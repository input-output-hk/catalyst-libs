use crate::{
    CatalystSignedDocument, ContentEncoding, ContentType,
    builder::{Builder, ed25519::Ed25519SigningKey},
    catalyst_id::CatalystId,
    doc_types,
    uuid::UuidV7,
};

pub fn contest_ballot_doc(
    linked: &CatalystSignedDocument,
    parameters: &CatalystSignedDocument,
    sk: &Ed25519SigningKey,
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

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Cbor,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::CONTEST_BALLOT.clone(),
            "id": id,
            "ver": ver,
            "ref": [linked_ref],
            "parameters": [parameters_ref],
        }))?
        .with_raw_cbor_content(&[160])?
        .add_signature(|m| sk.sign(&m), kid)?
        .build()
}
