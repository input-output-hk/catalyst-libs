use ed25519_dalek::{SigningKey, ed25519::signature::Signer};

use crate::{
    CatalystSignedDocument, ContentEncoding, ContentType, builder::Builder,
    catalyst_id::CatalystId, doc_types, uuid::UuidV7,
};

pub fn proposal_form_template_doc(
    content: &serde_json::Value,
    parameters: &CatalystSignedDocument,
    sk: SigningKey,
    kid: CatalystId,
    id: Option<UuidV7>,
) -> anyhow::Result<CatalystSignedDocument> {
    let (id, ver) = id.map(|v| (v, UuidV7::new())).unwrap_or_else(|| {
        let id = UuidV7::new();
        (id, id)
    });
    let parameters_ref = parameters.doc_ref()?;

    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::SchemaJson,
            "content-encoding": ContentEncoding::Brotli,
            "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
            "id": id,
            "ver": ver,
            "parameters": [parameters_ref],
        }))?
        .with_json_content(content)?
        .add_signature(|m| sk.sign(&m).to_vec(), kid)?
        .build()
}
