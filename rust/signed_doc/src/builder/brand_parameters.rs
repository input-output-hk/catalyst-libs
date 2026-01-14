use ed25519_dalek::{SigningKey, ed25519::signature::Signer};

use crate::{
    CatalystSignedDocument, ContentEncoding, ContentType, builder::Builder,
    catalyst_id::CatalystId, doc_types, uuid::UuidV7,
};

pub fn brand_parameters_doc(
    content: &serde_json::Value,
    template: &CatalystSignedDocument,
    sk: SigningKey,
    kid: CatalystId,
    id: Option<UuidV7>,
) -> anyhow::Result<CatalystSignedDocument> {
    let (id, ver) = id.map(|v| (v, UuidV7::new())).unwrap_or_else(|| {
        let id = UuidV7::new();
        (id, id)
    });
    let template_ref = template.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "content-encoding": ContentEncoding::Brotli,
            "id": id,
            "ver": ver,
            "type": doc_types::BRAND_PARAMETERS.clone(),
            "template": [template_ref],
        }))?
        .with_json_content(content)?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
