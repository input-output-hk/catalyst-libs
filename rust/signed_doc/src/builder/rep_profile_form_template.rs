use crate::{
    CatalystSignedDocument, ContentEncoding, ContentType,
    builder::{Builder, ed25519::Ed25519SigningKey},
    catalyst_id::CatalystId,
    doc_types,
    uuid::UuidV7,
};

// TODO: use doc_builder! macro
pub fn rep_profile_form_template_doc(
    content: &serde_json::Value,
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
    let parameters_ref = parameters.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::SchemaJson,
            "content-encoding": ContentEncoding::Brotli,
            "id": id,
            "ver": ver,
            "type": doc_types::REP_PROFILE_FORM_TEMPLATE.clone(),
            "parameters": [parameters_ref]
        }))?
        .with_json_content(content)?
        .add_signature(|m| sk.sign(&m), kid)?
        .build()
}
