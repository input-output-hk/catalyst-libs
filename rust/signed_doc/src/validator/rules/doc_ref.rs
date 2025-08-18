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
        exp_ref_types: Vec<DocType>,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'ref' is not specified
    NotSpecified,
}
impl RefRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let context: &str = "Ref rule check";
        if let Self::Specified {
            exp_ref_types,
            optional,
        } = self
        {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                let ref_validator = |ref_doc: CatalystSignedDocument| {
                    referenced_doc_check(&ref_doc, exp_ref_types, "ref", doc.report())
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
    ref_doc: &CatalystSignedDocument,
    exp_ref_types: &[DocType],
    field_name: &str,
    report: &ProblemReport,
) -> bool {
    let Ok(ref_doc_type) = ref_doc.doc_type() else {
        report.missing_field("type", "Referenced document must have type field");
        return false;
    };

    // Check that the type matches one of the expected ones
    if exp_ref_types
        .iter()
        .all(|exp_type| ref_doc_type != exp_type)
    {
        report.invalid_value(
            field_name,
            &ref_doc_type.to_string(),
            &exp_ref_types
                .iter()
                .fold(String::new(), |s, v| format!("{s}, {v}")),
            &format!("Invalid referenced document type, during validation of {field_name} field"),
        );
        return false;
    }
    true
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider, DocLocator, DocumentRef,
    };

    #[test_case(
        |exp_types, provider| {
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => true
        ;
        "valid reference to the one correct document"
    )]
    #[test_case(
        |exp_types, provider| {
            let ref_doc_1 = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
                .build();
            provider.add_document(None, &ref_doc_1).unwrap();
            let ref_doc_2 = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
                .build();
            provider.add_document(None, &ref_doc_2).unwrap();
            let ref_doc_3 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
            .build();
            provider.add_document(None, &ref_doc_3).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc_1.doc_id().unwrap(),
                        ref_doc_1.doc_ver().unwrap(),
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        ref_doc_2.doc_id().unwrap(),
                        ref_doc_2.doc_ver().unwrap(),
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        ref_doc_3.doc_id().unwrap(),
                        ref_doc_3.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => true
        ;
        "valid reference to the multiple documents"
    )]
    #[test_case(
        |exp_types, provider| {
            let ref_doc_1 = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
                .build();
            provider.add_document(None, &ref_doc_1).unwrap();
            let ref_doc_2 = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
                .build();
            provider.add_document(None, &ref_doc_2).unwrap();
            let ref_doc_3 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build();
            provider.add_document(None, &ref_doc_3).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc_1.doc_id().unwrap(),
                        ref_doc_1.doc_ver().unwrap(),
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        ref_doc_2.doc_id().unwrap(),
                        ref_doc_2.doc_ver().unwrap(),
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        ref_doc_3.doc_id().unwrap(),
                        ref_doc_3.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reference to the multiple documents, one of them invalid `type` field"
    )]
    #[test_case(
        |exp_types, provider| {
            let ref_doc_1 = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
                .build();
            provider.add_document(None, &ref_doc_1).unwrap();
            let ref_doc_2 = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[1].clone()))
                .build();
            provider.add_document(None, &ref_doc_2).unwrap();
            let ref_doc_3 = Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build();
            provider.add_document(None, &ref_doc_3).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc_1.doc_id().unwrap(),
                        ref_doc_1.doc_ver().unwrap(),
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        ref_doc_2.doc_id().unwrap(),
                        ref_doc_2.doc_ver().unwrap(),
                        DocLocator::default(),
                    ),
                    DocumentRef::new(
                        ref_doc_3.doc_id().unwrap(),
                        ref_doc_3.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reference to the multiple documents, one of them missing `type` field"
    )]
    #[test_case(
        |exp_types, provider| {
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_types[0].clone()))
                .build();
            provider.add_document(Some(DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default())), &ref_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "invalid reference to the document, which has different id and ver fields as stated in the `ref` field"
    )]
    #[test_case(
        |_, _| {
            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    ),
                    ]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reference to the missing one document"
    )]
    #[tokio::test]
    async fn ref_specified_test(
        doc_gen: impl FnOnce(
            &[DocType; 2],
            &mut TestCatalystSignedDocumentProvider,
        ) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

        let doc = doc_gen(&exp_types, &mut provider);

        let non_optional_res = RefRule::Specified {
            exp_ref_types: exp_types.to_vec(),
            optional: false,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        let optional_res = RefRule::Specified {
            exp_ref_types: exp_types.to_vec(),
            optional: true,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        assert_eq!(non_optional_res, optional_res);
        non_optional_res
    }

    #[tokio::test]
    async fn ref_specified_optional_test() {
        let provider = TestCatalystSignedDocumentProvider::default();
        let rule = RefRule::Specified {
            exp_ref_types: vec![UuidV4::new().into()],
            optional: true,
        };

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let provider = TestCatalystSignedDocumentProvider::default();
        let rule = RefRule::Specified {
            exp_ref_types: vec![UuidV4::new().into()],
            optional: false,
        };

        let doc = Builder::new().build();
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
