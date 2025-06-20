//! `parameters` rule type impl.

use super::doc_ref::referenced_doc_check;
use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument, DocType,
};

/// `parameters` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParametersRule {
    /// Is `parameters` specified
    Specified {
        /// expected `type` field of the parameter doc
        exp_parameters_type: Vec<DocType>,
        /// optional flag for the `parameters` field
        optional: bool,
    },
    /// `parameters` is not specified
    #[allow(unused)]
    NotSpecified,
}

impl ParametersRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        let context: &str = "Parameter rule check";
        if let Self::Specified {
            exp_parameters_type,
            optional,
        } = self
        {
            if let Some(parameters) = doc.doc_meta().parameters() {
                let parameters_validator = |ref_doc: CatalystSignedDocument| {
                    // Check that the type matches one of the expected ones
                    exp_parameters_type.iter().any(|exp_type| {
                        referenced_doc_check(&ref_doc, exp_type, "parameters", doc.report())
                    })
                };
                for dr in parameters.doc_refs() {
                    let result =
                        validate_provided_doc(dr, provider, doc.report(), parameters_validator)
                            .await?;
                    // Reference ALL of them
                    if !result {
                        return Ok(false);
                    }
                }
                return Ok(true);
            } else if !optional {
                doc.report().missing_field("parameters", context);
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if let Some(parameters) = doc.doc_meta().parameters() {
                doc.report().unknown_field(
                    "parameters",
                    &parameters.to_string(),
                    &format!("{context}, document does not expect to have a parameters field"),
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
#[allow(clippy::similar_names, clippy::too_many_lines)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider, Builder};

    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_parameters_cat_type = UuidV4::new();
        let exp_parameters_cam_type = UuidV4::new();
        let exp_parameters_brand_type = UuidV4::new();

        let exp_param_type: Vec<DocType> = vec![
            exp_parameters_cat_type.into(),
            exp_parameters_cam_type.into(),
            exp_parameters_brand_type.into(),
        ];

        let valid_category_doc_id = UuidV7::new();
        let valid_category_doc_ver = UuidV7::new();
        let valid_brand_doc_id = UuidV7::new();
        let valid_brand_doc_ver = UuidV7::new();
        let another_type_category_doc_id = UuidV7::new();
        let another_type_category_doc_ver = UuidV7::new();
        let missing_type_category_doc_id = UuidV7::new();
        let missing_type_category_doc_ver = UuidV7::new();

        // Prepare provider documents
        {
            // Category doc
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": valid_category_doc_id.to_string(),
                    "ver": valid_category_doc_ver.to_string(),
                    "type": exp_parameters_cat_type.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();

            // Brand doc
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": valid_brand_doc_id.to_string(),
                    "ver": valid_brand_doc_ver.to_string(),
                    "type": exp_parameters_cat_type.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();

            // Other type
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": another_type_category_doc_id.to_string(),
                    "ver": another_type_category_doc_ver.to_string(),
                    "type": UuidV4::new().to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();

            // Missing `type` field in the referenced document
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_type_category_doc_id.to_string(),
                    "ver": missing_type_category_doc_ver.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();
        }

        // Create a document where `parameters` field is required and referencing a valid document
        // in provider. Using doc ref of new implementation.
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_param_type.clone(),
            optional: false,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": 
                [{"id": valid_category_doc_id.to_string(), "ver": valid_category_doc_ver.to_string(), "cid": "0x" }]
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Parameters contain multiple ref
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": 
                [{"id": valid_category_doc_id.to_string(), "ver": valid_category_doc_ver.to_string(), "cid": "0x" },
                {"id": valid_brand_doc_id.to_string(), "ver": valid_brand_doc_ver.to_string(), "cid": "0x" }]
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Parameters contain multiple ref, but one of them is invalid (not registered).
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": 
                [{"id": valid_category_doc_id.to_string(), "ver": valid_category_doc_ver.to_string(), "cid": "0x" },
                {"id": UuidV7::new().to_string() , "ver": UuidV7::new().to_string(), "cid": "0x" }]
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Checking backward compatible
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters":
                {"id": valid_category_doc_id.to_string(), "ver": valid_category_doc_ver.to_string()}
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // All correct, `parameters` field is missing, but its optional
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_param_type.clone(),
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Missing `parameters` field, but its required
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_param_type,
            optional: false,
        };
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Reference to the document with another `type` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": {"id": another_type_category_doc_id.to_string(), "ver": another_type_category_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Missing `type` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": {"id": missing_type_category_doc_id.to_string(), "ver": missing_type_category_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Cannot find a referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": {"id": UuidV7::new().to_string(), "ver": UuidV7::new().to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn parameters_rule_not_specified_test() {
        let rule = ParametersRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"parameters": {"id": ref_id.to_string(), "ver": ref_ver.to_string() } }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
