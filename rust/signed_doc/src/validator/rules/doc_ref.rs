//! `ref` rule type impl.

use std::collections::HashMap;

use catalyst_types::problem_report::ProblemReport;

use crate::{
    providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocType, DocumentRef,
    DocumentRefs,
};

/// `ref` field validation rule
#[derive(Debug)]
pub(crate) enum RefRule {
    /// Is 'ref' specified
    Specified {
        /// allowed `type` field of the referenced doc
        allowed_type: Vec<DocType>,
        /// allows multiple document references or only one
        multiple: bool,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'ref' is not specified
    NotSpecified,
}
impl RefRule {
    /// Generating `RefRule` from specs
    pub(crate) fn new(
        docs: &HashMap<catalyst_signed_doc_spec::DocumentName, catalyst_signed_doc_spec::DocSpec>,
        spec: &catalyst_signed_doc_spec::metadata::doc_ref::Ref,
    ) -> anyhow::Result<Self> {
        let optional = match spec.required {
            catalyst_signed_doc_spec::is_required::IsRequired::Yes => false,
            catalyst_signed_doc_spec::is_required::IsRequired::Optional => true,
            catalyst_signed_doc_spec::is_required::IsRequired::Excluded => {
                anyhow::ensure!(
                    spec.doc_type.is_empty() && spec.multiple.is_none(),
                     "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'ref' metadata definition"
                );
                return Ok(Self::NotSpecified);
            },
        };

        anyhow::ensure!(!spec.doc_type.is_empty(), "'type' field should exists and has at least one entry for the required 'ref' metadata definition");

        let exp_ref_types = spec.doc_type.iter().try_fold(
            Vec::new(),
            |mut res, doc_name| -> anyhow::Result<_> {
                let docs_spec = docs.get(doc_name).ok_or(anyhow::anyhow!(
                    "cannot find a document definition {doc_name}"
                ))?;
                res.push(docs_spec.doc_type.as_str().parse()?);
                Ok(res)
            },
        )?;

        let multiple = spec.multiple.ok_or(anyhow::anyhow!(
            "'multiple' field should exists for the required 'ref' metadata definition"
        ))?;

        Ok(Self::Specified {
            allowed_type: exp_ref_types,
            multiple,
            optional,
        })
    }

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
            allowed_type: exp_ref_types,
            multiple,
            optional,
        } = self
        {
            if let Some(doc_refs) = doc.doc_meta().doc_ref() {
                return doc_refs_check(
                    doc_refs,
                    exp_ref_types,
                    *multiple,
                    "ref",
                    provider,
                    doc.report(),
                    |_| true,
                )
                .await;
            } else if !optional {
                doc.report()
                    .missing_field("ref", &format!("{context}, document must have ref field"));
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
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

/// Validate all the document references by the defined validation rules,
/// plus conducting additional validations with the provided `validator`.
/// Document all possible error in doc report (no fail fast)
pub(crate) async fn doc_refs_check<Provider, Validator>(
    doc_refs: &DocumentRefs,
    exp_ref_types: &[DocType],
    multiple: bool,
    field_name: &str,
    provider: &Provider,
    report: &ProblemReport,
    validator: Validator,
) -> anyhow::Result<bool>
where
    Provider: CatalystSignedDocumentProvider,
    Validator: Fn(&CatalystSignedDocument) -> bool,
{
    let mut all_valid = true;

    if !multiple && doc_refs.len() > 1 {
        report.other(
            format!(
                "Only ONE document reference is allowed, found {} document references",
                doc_refs.len()
            )
            .as_str(),
            &format!("Referenced document validation for the `{field_name}` field"),
        );
        return Ok(false);
    }

    for dr in doc_refs.iter() {
        if let Some(ref ref_doc) = provider.try_get_doc(dr).await? {
            let is_valid = referenced_doc_type_check(ref_doc, exp_ref_types, field_name, report)
                && referenced_doc_id_and_ver_check(ref_doc, dr, field_name, report)
                && validator(ref_doc);

            if !is_valid {
                all_valid = false;
            }
        } else {
            report.functional_validation(
                &format!("Cannot retrieve a document {dr}"),
                &format!("Referenced document validation for the `{field_name}` field"),
            );
            all_valid = false;
        }
    }
    Ok(all_valid)
}

/// Validation check that the provided `ref_doc` is a correct referenced document found by
/// `original_doc_ref`
fn referenced_doc_id_and_ver_check(
    ref_doc: &CatalystSignedDocument,
    original_doc_ref: &DocumentRef,
    field_name: &str,
    report: &ProblemReport,
) -> bool {
    let Ok(id) = ref_doc.doc_id() else {
        report.missing_field(
            "id",
            &format!("Referenced document validation for the `{field_name}` field"),
        );
        return false;
    };

    let Ok(ver) = ref_doc.doc_ver() else {
        report.missing_field(
            "ver",
            &format!("Referenced document validation for the `{field_name}` field"),
        );
        return false;
    };

    // id and version must match the values in ref doc
    if &id != original_doc_ref.id() && &ver != original_doc_ref.ver() {
        report.invalid_value(
            "id and version",
            &format!("id: {id}, ver: {ver}"),
            &format!(
                "id: {}, ver: {}",
                original_doc_ref.id(),
                original_doc_ref.ver()
            ),
            &format!("Referenced document validation for the `{field_name}` field"),
        );
        return false;
    }

    true
}

/// Validation check that the provided `ref_doc` has an expected `type` field value from
/// the allowed  `exp_ref_types` list
fn referenced_doc_type_check(
    ref_doc: &CatalystSignedDocument,
    exp_ref_types: &[DocType],
    field_name: &str,
    report: &ProblemReport,
) -> bool {
    let Ok(ref_doc_type) = ref_doc.doc_type() else {
        report.missing_field(
            "type",
            &format!("Document reference validation for the `{field_name}` field"),
        );
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
        builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
        DocLocator, DocumentRef,
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
    async fn ref_multiple_specified_test(
        doc_gen: impl FnOnce(&[DocType; 2], &mut TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystProvider::default();

        let exp_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

        let doc = doc_gen(&exp_types, &mut provider);

        let non_optional_res = RefRule::Specified {
            allowed_type: exp_types.to_vec(),
            multiple: true,
            optional: false,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        let optional_res = RefRule::Specified {
            allowed_type: exp_types.to_vec(),
            multiple: true,
            optional: true,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        assert_eq!(non_optional_res, optional_res);
        non_optional_res
    }

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
        "valid document with a single reference"
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
        => false
        ;
        "valid document with multiple references"
    )]
    #[tokio::test]
    async fn ref_non_multiple_specified_test(
        doc_gen: impl FnOnce(&[DocType; 2], &mut TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystProvider::default();

        let exp_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

        let doc = doc_gen(&exp_types, &mut provider);

        let non_optional_res = RefRule::Specified {
            allowed_type: exp_types.to_vec(),
            multiple: false,
            optional: false,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        let optional_res = RefRule::Specified {
            allowed_type: exp_types.to_vec(),
            multiple: false,
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
        let provider = TestCatalystProvider::default();
        let rule = RefRule::Specified {
            allowed_type: vec![UuidV4::new().into()],
            multiple: true,
            optional: true,
        };

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let provider = TestCatalystProvider::default();
        let rule = RefRule::Specified {
            allowed_type: vec![UuidV4::new().into()],
            multiple: true,
            optional: false,
        };

        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn ref_rule_not_specified_test() {
        let rule = RefRule::NotSpecified;
        let provider = TestCatalystProvider::default();

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
