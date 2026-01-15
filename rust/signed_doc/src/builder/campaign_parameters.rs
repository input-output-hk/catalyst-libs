use crate::{
    CatalystSignedDocument, ContentEncoding, ContentType,
    builder::{Builder, ed25519::Ed25519SigningKey},
    catalyst_id::CatalystId,
    doc_types,
    uuid::UuidV7,
};

pub fn campaign_parameters_doc(
    content: &serde_json::Value,
    template: &CatalystSignedDocument,
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
    let template_ref = template.doc_ref()?;
    let parameters_ref = parameters.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "id": id,
            "ver": ver,
            "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
            "template": [template_ref],
            "parameters": [parameters_ref]
        }))?
        .with_json_content(content)?
        .add_signature(|m| sk.sign(&m), kid)?
        .build()
}
