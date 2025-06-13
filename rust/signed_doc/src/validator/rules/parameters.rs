//! `parameters` rule type impl.

use super::doc_ref::referenced_doc_check;
use crate::{
    metadata::DocType, providers::CatalystSignedDocumentProvider,
    validator::utils::validate_provided_doc, CatalystSignedDocument,
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
    NotSpecified,
}

impl ParametersRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_parameters_type,
            optional,
        } = self
        {
            if let Some(parameters) = doc.doc_meta().parameters() {
                let parameters_validator = |replied_doc: CatalystSignedDocument| {
                    // For "parameters", it should be one of these types
                    exp_parameters_type.iter().any(|exp_type| {
                        referenced_doc_check(&replied_doc, exp_type, "parameters", doc.report())
                    })
                };
                return validate_provided_doc(
                    &parameters,
                    provider,
                    doc.report(),
                    parameters_validator,
                )
                .await;
            } else if !optional {
                doc.report()
                    .missing_field("parameters", "Document must have a parameters field");
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if let Some(parameters) = doc.doc_meta().parameters() {
                doc.report().unknown_field(
                    "parameters",
                    &parameters.to_string(),
                    "Document does not expect to have a parameters field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider, Builder};

    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        // Parameter types can contains multiple, but should match one of it.
        let exp_parameters_type: Vec<DocType> = vec![vec![UuidV4::new()].try_into().unwrap()];

        let valid_category_doc_id = UuidV7::new();
        let valid_category_doc_ver = UuidV7::new();
        let another_type_category_doc_id = UuidV7::new();
        let another_type_category_doc_ver = UuidV7::new();
        let missing_type_category_doc_id = UuidV7::new();
        let missing_type_category_doc_ver = UuidV7::new();

        // prepare replied documents
        {
            let exp_parameters_type_str: Vec<String> = exp_parameters_type
                .iter()
                .map(|dt| dt.to_string())
                .collect();

            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": valid_category_doc_id.to_string(),
                    "ver": valid_category_doc_ver.to_string(),
                    "type": exp_parameters_type_str,
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // reply doc with other `type` field
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": another_type_category_doc_id.to_string(),
                    "ver": another_type_category_doc_ver.to_string(),
                    "type": UuidV4::new().to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_type_category_doc_id.to_string(),
                    "ver": missing_type_category_doc_ver.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_parameters_type.clone().into(),
            optional: false,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": {"id": valid_category_doc_id.to_string(), "ver": valid_category_doc_ver }
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `parameters` field is missing, but its optional
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_parameters_type.clone().into(),
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `parameters` field, but its required
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_parameters_type.into(),
            optional: false,
        };
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": {"id": another_type_category_doc_id.to_string(), "ver": another_type_category_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": {"id": missing_type_category_doc_id.to_string(), "ver": missing_type_category_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
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
