use super::*;

pub fn category_parameters_doc() -> anyhow::Result<CatalystSignedDocument> {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::CATEGORY_PARAMETERS.clone(),
        }))?
        .empty_content()?
        .build()
}
