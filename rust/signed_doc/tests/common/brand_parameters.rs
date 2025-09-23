use super::*;

pub fn brand_parameters_doc() -> anyhow::Result<CatalystSignedDocument> {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json,
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::BRAND_PARAMETERS.clone(),
        }))?
        .empty_content()?
        .build()
}
