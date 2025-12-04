//! `parameters` rule type impl.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use catalyst_signed_doc_spec::{
    DocSpecs, is_required::IsRequired, metadata::parameters::Parameters,
};
use catalyst_types::problem_report::ProblemReport;
use futures::FutureExt;

use crate::{
    CatalystSignedDocument, DocType, DocumentRef, DocumentRefs,
    providers::CatalystSignedDocumentProvider, validator::rules::doc_ref::doc_refs_check,
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

/// Performs a parameter link validation between a given reference field and the expected
/// parameters.
///
/// Validates that all referenced documents
/// have matching `parameters` with the current document's expected `exp_parameters`.
///
/// # Returns
/// - `Ok(true)` if:
///   - `ref_field` is `None`, or
///   - all referenced documents are successfully retrieved **and** each has a
///     `parameters` field that matches `exp_parameters`.
///
/// - `Ok(false)` if:
///   - any referenced document cannot be retrieved,
///   - a referenced document is missing its `parameters` field, or
///   - the parameters mismatch the expected ones.
///
/// - `Err(anyhow::Error)` if an unexpected error occurs while accessing the provider.
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

    // collect recursive lineage of expected parameters
    let mut allowed_params: HashSet<DocumentRef> = HashSet::new();

    for exp_doc_ref in exp_parameters.iter() {
        let lineage = collect_parameter_lineage(exp_doc_ref, field_name, provider, report).await?;
        allowed_params.extend(lineage);
    }

    let mut all_valid = true;

    for doc_ref in ref_field.iter() {
        let Some(referred_doc) = provider.try_get_doc(doc_ref).await? else {
            report.functional_validation(
                &format!("Cannot retrieve a document {doc_ref}"),
                &format!("Referenced document link validation for `{field_name}`"),
            );
            all_valid = false;
            continue;
        };

        let Some(referred_doc_params) = referred_doc.doc_meta().parameters() else {
            report.missing_field(
                "parameters",
                &format!(
                    "Referenced document via `{field_name}` must have `parameters`. Doc: {referred_doc}"
                ),
            );
            all_valid = false;
            continue;
        };

        // reference parameters must be subset of allowed params
        let mut valid = true;
        for param_ref in referred_doc_params.iter() {
            if !allowed_params.contains(param_ref) {
                valid = false;
                break;
            }
        }

        if !valid {
            report.invalid_value(
                "parameters",
                &format!("Reference doc parameters: {referred_doc_params}"),
                &format!("Allowed recursive hierarchy: {:?}", allowed_params),
                &format!(
                    "Referenced document via `{field_name}` must have parameters within the recursive lineage"
                ),
            );
            all_valid = false;
        }
    }

    Ok(all_valid)
}

/// Recursively collects the full parameter lineage for a parameter document.
async fn collect_parameter_lineage<Provider>(
    root: &DocumentRef,
    field_name: &str,
    provider: &Provider,
    report: &ProblemReport,
) -> anyhow::Result<HashSet<DocumentRef>>
where
    Provider: CatalystSignedDocumentProvider,
{
    let mut visited = HashSet::new();
    let mut stack = vec![root.clone()];

    while let Some(current) = stack.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }

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
        }
    }

    Ok(visited)
}
