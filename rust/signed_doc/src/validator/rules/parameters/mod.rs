//! `parameters` rule type impl.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use catalyst_signed_doc_spec::{
    DocSpecs, is_required::IsRequired, metadata::parameters::Parameters,
};
use catalyst_types::problem_report::ProblemReport;
use futures::FutureExt;
use itertools::Itertools;

use crate::{
    CatalystSignedDocument, DocType, DocumentRef, DocumentRefs,
    providers::{CatalystSignedDocumentProvider, Provider},
    validator::{CatalystSignedDocumentValidationRule, rules::doc_ref::doc_refs_check},
};

/// `parameters` field validation rule
#[derive(Debug)]
pub(crate) enum ParametersRule {
    /// Is `parameters` specified
    Specified {
        /// expected `type` field of the parameter doc
        allowed_type: Vec<DocType>,
        /// optional flag for the `parameters` field
        optional: bool,
    },
    /// `parameters` is not specified
    NotSpecified,
}

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for ParametersRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider).await
    }
}

impl ParametersRule {
    /// Generating `ParametersRule` from specs
    pub(crate) fn new(
        docs: &DocSpecs,
        spec: &Parameters,
    ) -> anyhow::Result<Self> {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                anyhow::ensure!(
                    spec.doc_type.is_empty() && !spec.multiple,
                    "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'parameters'  metadata definition"
                );
                return Ok(Self::NotSpecified);
            },
        };

        anyhow::ensure!(
            !spec.doc_type.is_empty(),
            "'type' field should exists and has at least one entry for the required 'parameters' metadata definition"
        );
        anyhow::ensure!(
            !spec.multiple,
            "'multiple' field should be only set to false for the required 'parameters' metadata definition"
        );

        let allowed_type = spec.doc_type.iter().try_fold(
            Vec::new(),
            |mut res, doc_name| -> anyhow::Result<_> {
                let docs_spec = docs.get(doc_name).ok_or(anyhow::anyhow!(
                    "cannot find a document definition {doc_name}"
                ))?;
                res.push(docs_spec.doc_type.as_str().parse()?);
                Ok(res)
            },
        )?;

        Ok(Self::Specified {
            allowed_type,
            optional,
        })
    }

    /// Field validation rule
    async fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        let context: &str = "Parameter rule check";
        if let Self::Specified {
            allowed_type: exp_parameters_type,
            optional,
        } = self
        {
            if let Some(parameters_ref) = doc.doc_meta().parameters() {
                let parameters_check = doc_refs_check(
                    parameters_ref,
                    exp_parameters_type,
                    false,
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
                let chain_field = doc
                    .doc_meta()
                    .chain()
                    .and_then(|v| v.document_ref().cloned())
                    .map(|v| vec![v].into());
                let chain_link_check = link_check(
                    chain_field.as_ref(),
                    parameters_ref,
                    "chain",
                    provider,
                    doc.report(),
                )
                .boxed();

                let checks = [
                    parameters_check,
                    template_link_check,
                    ref_link_check,
                    reply_link_check,
                    chain_link_check,
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
        if let Self::NotSpecified = self
            && let Some(parameters) = doc.doc_meta().parameters()
        {
            doc.report().unknown_field(
                "parameters",
                &parameters.to_string(),
                &format!("{context}, document does not expect to have a parameters field"),
            );
            return Ok(false);
        }

        Ok(true)
    }
}

/// Validates that all documents referenced by `ref_field` recursively contain
/// `parameters` matching the expected `exp_parameters`.
///
/// The check expands each referenced document's parameter chain and succeeds
/// if any discovered parameter set equals `exp_parameters`.
///
/// Returns:
/// - `Ok(true)` if `ref_field` is `None` or yield a matching parameter set.
/// - `Ok(false)` if no recursive parameter set matches the expected one.
/// - `Err` if an unexpected provider error occurs.
pub(crate) async fn link_check(
    ref_field: Option<&DocumentRefs>,
    exp_parameters: &DocumentRefs,
    field_name: &str,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<bool> {
    let Some(ref_field) = ref_field else {
        return Ok(true);
    };

    let mut allowed_params = HashSet::new();
    let mut all_valid = true;
    for doc_ref in exp_parameters.iter() {
        let (valid, result) =
            collect_parameters_recursively(doc_ref, field_name, provider, report).await?;
        all_valid &= valid;
        allowed_params.extend(result);
    }

    for doc_ref in ref_field.iter() {
        if let Some(referenced_doc) = provider.try_get_doc(doc_ref).await? {
            if let Some(ref_params) = referenced_doc.doc_meta().parameters() {
                if !ref_params.iter().all(|v| allowed_params.contains(v)) {
                    report.invalid_value(
                        "parameters",
                        &format!("[{}]", ref_params.iter().map(ToString::to_string).join(",")),
                        &format!("[{}]", allowed_params.iter().map(ToString::to_string).join(",")),
                        &format!("Referenced document {doc_ref} via {field_name} `parameters` field must match one of the allowed params"),
                    );
                    all_valid = false;
                }
            } else {
                report.missing_field(
                    "'parameters'",
                    &format!("Referenced document {doc_ref} must have `parameters` field"),
                );
                all_valid = false;
            }
        } else {
            report.functional_validation(
                &format!("Cannot retrieve a document {doc_ref}"),
                &format!("Referenced document link validation for `{field_name}`"),
            );
            all_valid = false;
        }
    }

    Ok(all_valid)
}

/// Recursively traverses the parameter chain starting from a given `root` document
/// reference, collecting all discovered `parameters` sets.
///
/// Returns:
/// - `(true, set)` if all referenced documents are retrievable.
/// - `(false, set)` if any underlying document cannot be fetched.
///
/// All encountered parameter lists are returned; traversal is cycle-safe
/// and explores deeper parameter references recursively.
async fn collect_parameters_recursively(
    root: &DocumentRef,
    field_name: &str,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<(bool, HashSet<DocumentRef>)> {
    let mut all_valid = true;
    let mut result: HashSet<_> = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = vec![root.clone()];

    while let Some(current) = stack.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        result.insert(current.clone());

        if let Some(doc) = provider.try_get_doc(&current).await? {
            if let Some(params) = doc.doc_meta().parameters() {
                for param in params.iter() {
                    if !visited.contains(param) {
                        stack.push(param.clone());
                    }
                }
            }
        } else {
            report.functional_validation(
                &format!("Cannot retrieve a document {current}"),
                &format!("Referenced document link validation for `{field_name}`"),
            );
            all_valid = false;
        }

        result.insert(current);
    }

    Ok((all_valid, result))
}
