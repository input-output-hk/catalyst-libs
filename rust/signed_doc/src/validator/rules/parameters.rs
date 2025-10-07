//! `parameters` rule type impl.

use catalyst_types::{problem_report::ProblemReport, uuid::UuidV4};
use futures::FutureExt;

use super::doc_ref::referenced_doc_check;
use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument, DocumentRef,
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
    #[allow(dead_code)]
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
        if let Self::Specified {
            exp_parameters_type,
            optional,
        } = self
        {
            if let Some(ref parameters) = doc.doc_meta().parameters() {
                let parameters_validator = |replied_doc: CatalystSignedDocument| {
                    referenced_doc_check(
                        &replied_doc,
                        exp_parameters_type.uuid(),
                        "parameters",
                        doc.report(),
                    )
                };
                let parameters_check =
                    validate_provided_doc(parameters, provider, doc.report(), parameters_validator)
                        .boxed();

                let template = doc.doc_meta().template();
                let template_link_check = link_check(
                    template.as_ref(),
                    parameters,
                    "template",
                    provider,
                    doc.report(),
                )
                .boxed();
                let doc_ref = doc.doc_meta().doc_ref();
                let ref_link_check =
                    link_check(doc_ref.as_ref(), parameters, "ref", provider, doc.report()).boxed();
                let reply = doc.doc_meta().reply();
                let reply_link_check =
                    link_check(reply.as_ref(), parameters, "reply", provider, doc.report()).boxed();

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

/// Parameter Link reference check
#[allow(dead_code)]
pub(crate) async fn link_check<Provider>(
    ref_field: Option<&DocumentRef>,
    exp_parameters: &DocumentRef,
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

    if let Some(ref ref_doc) = provider.try_get_doc(ref_field).await? {
        let Some(ref_doc_parameters) = ref_doc.doc_meta().parameters() else {
            report.missing_field(
                    "parameters",
                    &format!(
                        "Referenced document via {field_name} must have `parameters` field. Referenced Document: {ref_doc}"
                    ),
                );
            return Ok(false);
        };

        if exp_parameters == &ref_doc_parameters {
            Ok(true)
        } else {
            report.invalid_value(
                    "parameters",
                    &format!("Reference doc param: {ref_doc_parameters}",),
                    &format!("Doc param: {exp_parameters}"),
                    &format!(
                        "Referenced document via {field_name} `parameters` field must match. Referenced Document: {ref_doc}"
                    ),
                );

            Ok(false)
        }
    } else {
        report.functional_validation(
            &format!("Cannot retrieve a document {ref_field}"),
            &format!("Referenced document link validation for the `{field_name}` field"),
        );
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};
    use test_case::test_case;

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider, Builder};

    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(), 
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let template_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build();
            provider.add_document(template_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "template": {
                        "id": template_doc.doc_id().unwrap(),
                        "ver": template_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document, with valid template field"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "template": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing template doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let template_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                }))
                .unwrap()
                .build();
            provider.add_document(template_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "template": {
                        "id": template_doc.doc_id().unwrap(),
                        "ver": template_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing parameters field in template doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let template_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "parameters": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    }
                }))
                .unwrap()
                .build();
            provider.add_document(template_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "template": {
                        "id": template_doc.doc_id().unwrap(),
                        "ver": template_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with different parameters field in template doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let replied_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build();
            provider.add_document(replied_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "reply": {
                        "id": replied_doc.doc_id().unwrap(),
                        "ver": replied_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document, with valid reply field"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "reply": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing reply doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let reply_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                }))
                .unwrap()
                .build();
            provider.add_document(reply_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "reply": {
                        "id": reply_doc.doc_id().unwrap(),
                        "ver": reply_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing parameters field in replied doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let reply_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "parameters": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    }
                }))
                .unwrap()
                .build();
            provider.add_document(reply_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "reply": {
                        "id": reply_doc.doc_id().unwrap(),
                        "ver": reply_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with different parameters field in reply doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": {
                        "id": ref_doc.doc_id().unwrap(),
                        "ver": ref_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => true
        ;
        "valid reference to the valid parameters document, with valid ref field"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing ref doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": {
                        "id": ref_doc.doc_id().unwrap(),
                        "ver": ref_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with missing parameters field in ref doc"
    )]
    #[test_case(
        |exp_param_types, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": exp_param_types,
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            let ref_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "parameters": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    }
                }))
                .unwrap()
                .build();
            provider.add_document(ref_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "ref": {
                        "id": ref_doc.doc_id().unwrap(),
                        "ver": ref_doc.doc_ver().unwrap(),
                    },
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the valid parameters document, with different parameters field in ref doc"
    )]
    #[test_case(
        |_, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "type": UuidV4::new(),
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the invalid parameters document, wrong parameters type field value"
    )]
    #[test_case(
        |_, provider| {
            let parameter_doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                }))
                .unwrap()
                .build();
            provider.add_document(parameter_doc.clone()).unwrap();

            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "parameters": {
                        "id": parameter_doc.doc_id().unwrap(),
                        "ver": parameter_doc.doc_ver().unwrap(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "valid reference to the invalid parameters document, missing type field"
    )]
    #[test_case(
        |_, _| {
            Builder::new()
                .with_json_metadata(serde_json::json!({
                    "parameters": {
                        "id": UuidV7::new(),
                        "ver": UuidV7::new(),
                    }
                }))
                .unwrap()
                .build()
        }
        => false
        ;
        "reference to the not known document"
    )]
    #[tokio::test]
    async fn parameter_specified_test(
        doc_gen: impl FnOnce(UuidV4, &mut TestCatalystSignedDocumentProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let exp_parameters_type = UuidV4::new();

        let doc = doc_gen(exp_parameters_type, &mut provider);

        let non_optional_res = ParametersRule::Specified {
            exp_parameters_type,
            optional: false,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        let optional_res = ParametersRule::Specified {
            exp_parameters_type,
            optional: true,
        }
        .check(&doc, &provider)
        .await
        .unwrap();

        assert_eq!(non_optional_res, optional_res);
        non_optional_res
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
