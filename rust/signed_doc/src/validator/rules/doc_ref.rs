//! `ref` rule type impl.

use catalyst_types::problem_report::ProblemReport;

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_doc_refs,
    CatalystSignedDocument, DocType,
};

/// `ref` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RefRule {
    /// Is 'ref' specified
    Specified {
        /// expected `type` field of the referenced doc
        exp_ref_type: DocType,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'ref' is not specified
    NotSpecified,
}
impl RefRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        let context: &str = "Ref rule check";
        if let Self::Specified {
            exp_ref_type,
            optional,
        } = self
        {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                let ref_validator = |ref_doc: CatalystSignedDocument| {
                    referenced_doc_check(&ref_doc, exp_ref_type, "ref", doc.report())
                };
                return validate_doc_refs(doc_ref, provider, doc.report(), ref_validator).await;
            } else if !optional {
                doc.report()
                    .missing_field("ref", &format!("{context}, document must have ref field"));
                return Ok(false);
            }
        }
        if &Self::NotSpecified == self {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                doc.report().unknown_field(
                    "ref",
                    &doc_ref.to_string(),
                    &format!("{context}, document does not expect to have a ref field"),
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// A generic implementation of the referenced document validation.
pub(crate) fn referenced_doc_check(
    ref_doc: &CatalystSignedDocument, exp_ref_type: &DocType, field_name: &str,
    report: &ProblemReport,
) -> bool {
    let Ok(ref_doc_type) = ref_doc.doc_type() else {
        report.missing_field("type", "Referenced document must have type field");
        return false;
    };

    if ref_doc_type != exp_ref_type {
        report.invalid_value(
            field_name,
            &ref_doc_type.to_string(),
            &exp_ref_type.to_string(),
            "Invalid referenced document type",
        );
        return false;
    }
    true
}

#[cfg(test)]
#[allow(clippy::similar_names, clippy::too_many_lines)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider, DocLocator, DocumentRef,
    };

    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_ref_type = UuidV4::new();

        let valid_referenced_doc_id = UuidV7::new();
        let valid_referenced_doc_ver = UuidV7::new();
        let different_id_and_ver_referenced_doc_id = UuidV7::new();
        let different_id_and_ver_referenced_doc_ver = UuidV7::new();
        let another_type_referenced_doc_id = UuidV7::new();
        let another_type_referenced_doc_ver = UuidV7::new();
        let missing_type_referenced_doc_id = UuidV7::new();
        let missing_type_referenced_doc_ver = UuidV7::new();

        // Prepare provider documents
        {
            // Valid one
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(valid_referenced_doc_id))
                .with_metadata_field(SupportedField::Ver(valid_referenced_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_ref_type.into()))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Having different id and ver in registered reference
            let doc_ref = DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default());
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(different_id_and_ver_referenced_doc_id))
                .with_metadata_field(SupportedField::Ver(different_id_and_ver_referenced_doc_ver))
                .with_metadata_field(SupportedField::Type(exp_ref_type.into()))
                .build();
            provider.add_document(Some(doc_ref), &doc).unwrap();

            // Having another `type` field
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(another_type_referenced_doc_id))
                .with_metadata_field(SupportedField::Ver(another_type_referenced_doc_id))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Missing `type` field in the referenced document
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(missing_type_referenced_doc_id))
                .with_metadata_field(SupportedField::Ver(missing_type_referenced_doc_ver))
                .build();
            provider.add_document(None, &doc).unwrap();
        }

        // Create a document where `ref` field is required and referencing a valid document in
        // provider. Using doc ref of new implementation.
        let rule = RefRule::Specified {
            exp_ref_type: exp_ref_type.into(),
            optional: false,
        };
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    valid_referenced_doc_id,
                    valid_referenced_doc_ver,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Having multiple refs, where one ref doc is not found.
        // Checking match all of
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![
                    DocumentRef::new(
                        valid_referenced_doc_id,
                        valid_referenced_doc_ver,
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        different_id_and_ver_referenced_doc_id,
                        different_id_and_ver_referenced_doc_ver,
                        DocLocator::default(),
                    ),
                ]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Invalid the ref doc id and ver doesn't match the id and ver in ref doc ref
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    different_id_and_ver_referenced_doc_id,
                    different_id_and_ver_referenced_doc_ver,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // All correct, `ref` field is missing, but its optional
        let rule = RefRule::Specified {
            exp_ref_type: exp_ref_type.into(),
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Missing `ref` field, but its required
        let rule = RefRule::Specified {
            exp_ref_type: exp_ref_type.into(),
            optional: false,
        };
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Reference to the document with another `type` field
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    another_type_referenced_doc_id,
                    another_type_referenced_doc_ver,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // Missing `type` field in the referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    missing_type_referenced_doc_id,
                    missing_type_referenced_doc_ver,
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    DocLocator::default(),
                )]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn ref_rule_not_specified_test() {
        let rule = RefRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(ref_id, ref_ver, DocLocator::default())].into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
