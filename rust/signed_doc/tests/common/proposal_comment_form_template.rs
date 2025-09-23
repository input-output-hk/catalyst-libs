use super::*;

pub fn proposal_comment_form_template_doc(
    parameters_doc: &CatalystSignedDocument
) -> anyhow::Result<CatalystSignedDocument> {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::SchemaJson.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "parameters": {
                    "id": parameters_doc.doc_id()?,
                    "ver": parameters_doc.doc_ver()?,
                }
        }))?
        .with_json_content(&serde_json::json!({}))?
        .build()
}
