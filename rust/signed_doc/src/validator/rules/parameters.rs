//! `parameters` rule type impl.

use catalyst_types::uuid::UuidV4;

use super::doc_ref::referenced_doc_check;
use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// `parameters` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ParametersRule {
    /// Is `parameters` specified
    Specified {
        /// expected `type` field of the parameter doc
        exp_parameters_type: UuidV4,
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
                    if !referenced_doc_check(
                        &replied_doc,
                        exp_parameters_type.uuid(),
                        "parameters",
                        doc.report(),
                    ) {
                        return false;
                    }
                    let Some(doc_ref) = doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Document must have a ref field");
                        return false;
                    };

                    let Some(replied_doc_ref) = replied_doc.doc_meta().doc_ref() else {
                        doc.report()
                            .missing_field("ref", "Referenced document must have ref field");
                        return false;
                    };

                    if replied_doc_ref.id != doc_ref.id {
                        doc.report().invalid_value(
                                "parameters",
                                doc_ref.id .to_string().as_str(),
                                replied_doc_ref.id.to_string().as_str(),
                                "Invalid referenced document. Document ID should aligned with the replied document.",
                            );
                        return false;
                    }

                    true
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

    #[allow(clippy::too_many_lines)]
    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_parameters_type = UuidV4::new();
        let common_ref_id = UuidV7::new();
        let common_ref_ver = UuidV7::new();

        let valid_replied_doc_id = UuidV7::new();
        let valid_replied_doc_ver = UuidV7::new();
        let another_type_replied_doc_ver = UuidV7::new();
        let another_type_replied_doc_id = UuidV7::new();
        let missing_ref_replied_doc_ver = UuidV7::new();
        let missing_ref_replied_doc_id = UuidV7::new();
        let missing_type_replied_doc_ver = UuidV7::new();
        let missing_type_replied_doc_id = UuidV7::new();

        // prepare replied documents
        {
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                    "id": valid_replied_doc_id.to_string(),
                    "ver": valid_replied_doc_ver.to_string(),
                    "type": exp_parameters_type.to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // parameters doc with other `type` field
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                    "id": another_type_replied_doc_id.to_string(),
                    "ver": another_type_replied_doc_ver.to_string(),
                    "type": UuidV4::new().to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `ref` field in the referenced document
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": missing_ref_replied_doc_id.to_string(),
                    "ver": missing_ref_replied_doc_ver.to_string(),
                    "type": exp_parameters_type.to_string()
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                    "id": missing_type_replied_doc_id.to_string(),
                    "ver": missing_type_replied_doc_ver.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = ParametersRule::Specified {
            exp_parameters_type,
            optional: false,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                "parameters": { "id": valid_replied_doc_id.to_string(), "ver": valid_replied_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `parameters` field is missing, but its optional
        let rule = ParametersRule::Specified {
            exp_parameters_type,
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `parameters` field, but its required
        let rule = ParametersRule::Specified {
            exp_parameters_type,
            optional: false,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "parameters": { "id": valid_replied_doc_id.to_string(), "ver": valid_replied_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                "parameters": { "id": another_type_replied_doc_id.to_string(), "ver": another_type_replied_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                "parameters": { "id": missing_ref_replied_doc_id.to_string(), "ver": missing_type_replied_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
                "parameters": { "id": missing_type_replied_doc_id.to_string(), "ver": missing_type_replied_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // `ref` field does not align with the referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": UuidV7::new().to_string(), "ver": UuidV7::new().to_string() },
                "parameters": { "id": valid_replied_doc_id.to_string(), "ver": valid_replied_doc_ver.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": { "id": common_ref_id.to_string(), "ver": common_ref_ver.to_string() },
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
