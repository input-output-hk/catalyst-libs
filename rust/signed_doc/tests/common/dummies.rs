#![allow(clippy::unwrap_used)]

use std::sync::LazyLock;

use catalyst_signed_doc::*;

pub static BRAND_PARAMETERS_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::BRAND_PARAMETERS.clone(),
        }))
        .unwrap()
        .empty_content()
        .unwrap()
        .build()
        .unwrap()
});

pub static CAMPAIGN_PARAMETERS_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::CAMPAIGN_PARAMETERS.clone(),
        }))
        .unwrap()
        .empty_content()
        .unwrap()
        .build()
        .unwrap()
});

pub static CATEGORY_PARAMETERS_DOC: LazyLock<CatalystSignedDocument> = LazyLock::new(|| {
    Builder::new()
        .with_json_metadata(serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "id": UuidV7::new(),
            "ver": UuidV7::new(),
            "type": doc_types::CATEGORY_PARAMETERS.clone(),
        }))
        .unwrap()
        .empty_content()
        .unwrap()
        .build()
        .unwrap()
});

pub static PROPOSAL_TEMPLATE_FOR_BRAND_DOC: LazyLock<CatalystSignedDocument> =
    LazyLock::new(|| {
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
                "id": UuidV7::new(),
                "ver": UuidV7::new(),
                "parameters": {
                        "id": BRAND_PARAMETERS_DOC.doc_id().unwrap(),
                        "ver": BRAND_PARAMETERS_DOC.doc_ver().unwrap(),
                    },
            }))
            .unwrap()
            .with_json_content(&serde_json::json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }))
            .unwrap()
            .build()
            .unwrap()
    });

pub static PROPOSAL_TEMPLATE_FOR_CAMPAIGN_DOC: LazyLock<CatalystSignedDocument> =
    LazyLock::new(|| {
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
                "id": UuidV7::new(),
                "ver": UuidV7::new(),
                "parameters": {
                        "id": CAMPAIGN_PARAMETERS_DOC.doc_id().unwrap(),
                        "ver": CAMPAIGN_PARAMETERS_DOC.doc_ver().unwrap(),
                    },
            }))
            .unwrap()
            .with_json_content(&serde_json::json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }))
            .unwrap()
            .build()
            .unwrap()
    });

pub static PROPOSAL_TEMPLATE_FOR_CATEGORY_DOC: LazyLock<CatalystSignedDocument> =
    LazyLock::new(|| {
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "content-encoding": ContentEncoding::Brotli.to_string(),
                "type": doc_types::PROPOSAL_FORM_TEMPLATE.clone(),
                "id": UuidV7::new(),
                "ver": UuidV7::new(),
                "parameters": {
                        "id": CATEGORY_PARAMETERS_DOC.doc_id().unwrap(),
                        "ver": CATEGORY_PARAMETERS_DOC.doc_ver().unwrap(),
                    },
            }))
            .unwrap()
            .with_json_content(&serde_json::json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }))
            .unwrap()
            .build()
            .unwrap()
    });
