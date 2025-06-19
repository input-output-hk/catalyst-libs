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
        exp_parameters_type: DocType,
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
                    referenced_doc_check(
                        &replied_doc,
                        exp_parameters_type,
                        "parameters",
                        doc.report(),
                    )
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
    use crate::{
        metadata::SupportedField, providers::tests::TestCatalystSignedDocumentProvider, Builder,
        DocumentRef,
    };

    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_parameters_type = UuidV4::new();

        let valid_category_doc_id = UuidV7::new();
        let valid_category_doc_ver = UuidV7::new();
        let another_type_category_doc_id = UuidV7::new();
        let another_type_category_doc_ver = UuidV7::new();
        let missing_type_category_doc_id = UuidV7::new();
        let missing_type_category_doc_ver = UuidV7::new();

        // prepare replied documents
        {
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(valid_category_doc_id))
                .with_metadata_field(SupportedField::Ver(valid_category_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_parameters_type.into()))
                .build();
            provider.add_document(ref_doc).unwrap();

            // reply doc with other `type` field
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(another_type_category_doc_id))
                .with_metadata_field(SupportedField::Ver(another_type_category_doc_ver))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .build();
            provider.add_document(ref_doc).unwrap();

            // missing `type` field in the referenced document
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_type_category_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_type_category_doc_ver))
                .build();
            provider.add_document(ref_doc).unwrap();
        }

        // all correct
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_parameters_type.into(),
            optional: false,
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Parameters(DocumentRef {
                id: valid_category_doc_id,
                ver: valid_category_doc_ver,
            }))
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `parameters` field is missing, but its optional
        let rule = ParametersRule::Specified {
            exp_parameters_type: exp_parameters_type.into(),
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
            .with_metadata_field(SupportedField::Parameters(DocumentRef {
                id: another_type_category_doc_id,
                ver: another_type_category_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Parameters(DocumentRef {
                id: missing_type_category_doc_id,
                ver: missing_type_category_doc_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Parameters(DocumentRef {
                id: UuidV7::new(),
                ver: UuidV7::new(),
            }))
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
            .with_metadata_field(SupportedField::Parameters(DocumentRef {
                id: ref_id,
                ver: ref_ver,
            }))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
