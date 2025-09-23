use super::*;

pub fn proposal_form_template_doc(
    parameters_doc: &CatalystSignedDocument
) -> anyhow::Result<CatalystSignedDocument> {
    let id = UuidV7::new();
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::SchemaJson.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
            "id": id,
            "ver": id,
            "parameters": {
                    "id": parameters_doc.doc_id()?,
                    "ver": parameters_doc.doc_ver()?,
                },
        }))?
        .with_json_content(&serde_json::json!({}))?
        .build()
}
