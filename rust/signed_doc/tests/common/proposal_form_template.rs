use super::*;

pub fn proposal_form_template_doc(
    parameters_doc: &CatalystSignedDocument
) -> anyhow::Result<CatalystSignedDocument> {
    let doc = Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "parameters": {
                    "id": parameters_doc.doc_id()?,
                    "ver": parameters_doc.doc_ver()?,
                },
        }))?
        .with_json_content(&serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }))?
        .build()?;
    Ok(doc)
}
