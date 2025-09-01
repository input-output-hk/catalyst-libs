//! `parameters` rule type impl.

use catalyst_types::problem_report::ProblemReport;
use futures::FutureExt;

use crate::{
    providers::CatalystSignedDocumentProvider, validator::rules::doc_ref::doc_refs_check,
    CatalystSignedDocument, DocType, DocumentRefs,
};

/// `parameters` field validation rule
#[derive(Debug)]
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
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let context: &str = "Parameter rule check";
        if let Self::Specified {
            exp_parameters_type,
            optional,
        } = self
        {
            if let Some(parameters_ref) = doc.doc_meta().parameters() {
                let parameters_check = doc_refs_check(
                    parameters_ref,
                    exp_parameters_type,
                    "parameters",
                    provider,
                    doc.report(),
                    |_| true,
                )
                .boxed();

                let template_link_check = link_check(
                    doc.doc_meta().template(),
                    parameters_ref,
                    "template",
                    provider,
                    doc.report(),
                )
                .boxed();
                let ref_link_check = link_check(
                    doc.doc_meta().doc_ref(),
                    parameters_ref,
                    "ref",
                    provider,
                    doc.report(),
                )
                .boxed();
                let reply_link_check = link_check(
                    doc.doc_meta().reply(),
                    parameters_ref,
                    "reply",
                    provider,
                    doc.report(),
                )
                .boxed();

                let checks = [
                    parameters_check,
                    template_link_check,
                    ref_link_check,
                    reply_link_check,
                ];
                let res = futures::future::join_all(checks)
                    .await
                    .into_iter()
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .iter()
                    .all(|res| *res);

                return Ok(res);
            } else if !optional {
                doc.report().missing_field(
                    "parameters",
                    &format!("{context}, document must have parameters field"),
                );
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

/// Parameter Link reference check
pub(crate) async fn link_check<Provider>(
    ref_field: Option<&DocumentRefs>,
    exp_parameters: &DocumentRefs,
    field_name: &str,
    provider: &Provider,
    report: &ProblemReport,
) -> anyhow::Result<bool>
where
    Provider: CatalystSignedDocumentProvider,
{
    let Some(ref_field) = ref_field else {
        return Ok(true);
    };

    let mut all_valid = true;

    for dr in ref_field.iter() {
        if let Some(ref ref_doc) = provider.try_get_doc(dr).await? {
            let Some(ref_doc_parameters) = ref_doc.doc_meta().parameters() else {
                report.missing_field(
                    "parameters",
                    &format!(
                        "Referenced document via {field_name} must have `parameters` field. Referenced Document: {ref_doc}"
                    ),
                );
                all_valid = false;
                continue;
            };

            if exp_parameters != ref_doc_parameters {
                report.invalid_value(
                    "parameters",
                    &format!("Reference doc param: {ref_doc_parameters}",),
                    &format!("Doc param: {exp_parameters}"),
                    &format!(
                        "Referenced document via {field_name} `parameters` field must match. Referenced Document: {ref_doc}"
                    ),
                );
                all_valid = false;
            }
        } else {
            report.functional_validation(
                &format!("Cannot retrieve a document {dr}"),
                &format!("Referenced document link validation for the `{field_name}` field"),
            );
            all_valid = false;
        }
    }
    Ok(all_valid)
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
        DocLocator, DocumentRef,
    };

    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Parameters(
                    vec![DocumentRef::new(
                        parameter_doc.doc_id().unwrap(),
                        parameter_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let common_parameter_field: DocumentRefs = vec![DocumentRef::new(
                        parameter_doc.doc_id().unwrap(),
                        parameter_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into();
            let template_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Parameters(common_parameter_field.clone()))
                .build();
            provider.add_document(None, &template_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![DocumentRef::new(
                        template_doc.doc_id().unwrap(),
                        template_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(common_parameter_field))
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document, with valid template field"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing template doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let template_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .build();
            provider.add_document(None, &template_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![DocumentRef::new(
                        template_doc.doc_id().unwrap(),
                        template_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing parameters field in template doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let template_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    DocLocator::default(),
                )]
                .into()))
                .build();
            provider.add_document(None, &template_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Template(
                    vec![DocumentRef::new(
                        template_doc.doc_id().unwrap(),
                        template_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with different parameters field in template doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let common_parameter_field: DocumentRefs = vec![DocumentRef::new(
                        parameter_doc.doc_id().unwrap(),
                        parameter_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into();
            let replied_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Parameters(common_parameter_field.clone()))
                .build();
            provider.add_document(None, &replied_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        replied_doc.doc_id().unwrap(),
                        replied_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(common_parameter_field))
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document, with valid reply field"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing reply doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let reply_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .build();
            provider.add_document(None, &reply_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        reply_doc.doc_id().unwrap(),
                        reply_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing parameters field in replied doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let reply_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    DocLocator::default(),
                )]
                .into()))
                .build();
            provider.add_document(None, &reply_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Reply(
                    vec![DocumentRef::new(
                        reply_doc.doc_id().unwrap(),
                        reply_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with different parameters field in reply doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let common_parameter_field: DocumentRefs = vec![DocumentRef::new(
                        parameter_doc.doc_id().unwrap(),
                        parameter_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into();
            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Parameters(common_parameter_field.clone()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(common_parameter_field))
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document, with valid ref field"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing ref doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing parameters field in ref doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(exp_param_types[0].clone()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            let ref_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    UuidV7::new(),
                    UuidV7::new(),
                    DocLocator::default(),
                )]
                .into()))
                .build();
            provider.add_document(None, &ref_doc).unwrap();


            Builder::new()
                .with_metadata_field(SupportedField::Ref(
                    vec![DocumentRef::new(
                        ref_doc.doc_id().unwrap(),
                        ref_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into()
                ))
                .with_metadata_field(SupportedField::Parameters(vec![DocumentRef::new(
                    parameter_doc.doc_id().unwrap(),
                    parameter_doc.doc_ver().unwrap(),
                    DocLocator::default(),
                )]
                .into()))
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with different parameters field in ref doc"
    )]
    #[test_case(
        |_, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Parameters(
                    vec![DocumentRef::new(
                        parameter_doc.doc_id().unwrap(),
                        parameter_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reference to the invalid parameters document, wrong parameters type field value"
    )]
    #[test_case(
        |_, provider| {
            let parameter_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .build();
            provider.add_document(None, &parameter_doc).unwrap();

            Builder::new()
                .with_metadata_field(SupportedField::Parameters(
                    vec![DocumentRef::new(
                        parameter_doc.doc_id().unwrap(),
                        parameter_doc.doc_ver().unwrap(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "valid reference to the invalid parameters document, missing type field"
    )]
    #[test_case(
        |_, _| {
            Builder::new()
                .with_metadata_field(SupportedField::Parameters(
                    vec![DocumentRef::new(
                        UuidV7::new(),
                        UuidV7::new(),
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build()
        }
        => false
        ;
        "reference to the not known document"
    )]
    #[tokio::test]
    async fn parameter_specified_test(
        doc_gen: impl FnOnce(&[DocType; 2], &mut TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystProvider::default();

        let exp_param_types: [DocType; 2] = [UuidV4::new().into(), UuidV4::new().into()];

        let doc = doc_gen(&exp_param_types, &mut provider);

        let non_optional_res = ParametersRule::Specified {
            exp_parameters_type: exp_param_types.to_vec(),
            optional: false,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        let optional_res = ParametersRule::Specified {
            exp_parameters_type: exp_param_types.to_vec(),
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
        let rule = ParametersRule::Specified {
            exp_parameters_type: vec![UuidV4::new().into()],
            optional: true,
        };

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let provider = TestCatalystProvider::default();
        let rule = ParametersRule::Specified {
            exp_parameters_type: vec![UuidV4::new().into()],
            optional: false,
        };

        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn parameters_rule_not_specified_test() {
        let rule = ParametersRule::NotSpecified;
        let provider = TestCatalystProvider::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let ref_ver = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Parameters(
                vec![DocumentRef::new(ref_id, ref_ver, DocLocator::default())].into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
