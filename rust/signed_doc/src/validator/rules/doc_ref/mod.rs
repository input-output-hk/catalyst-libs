//! `ref` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{DocSpecs, is_required::IsRequired, metadata::doc_ref::Ref};
use catalyst_types::problem_report::ProblemReport;
use itertools::Itertools;

use crate::{
    CatalystSignedDocument, DocType, DocumentRef, DocumentRefs,
    providers::{CatalystSignedDocumentProvider, Provider},
    validator::CatalystSignedDocumentValidationRule,
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

impl CatalystSignedDocumentValidationRule for RefRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider)
    }
}

impl RefRule {
    /// Generating `RefRule` from specs
    pub(crate) fn new(
        docs: &DocSpecs,
        spec: &Ref,
    ) -> anyhow::Result<Self> {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                anyhow::ensure!(
                    spec.doc_type.is_empty() && !spec.multiple,
                    "'type' and 'multiple' fields could not been specified when 'required' is 'excluded' for 'ref' metadata definition"
                );
                return Ok(Self::NotSpecified);
            },
        };

        anyhow::ensure!(
            !spec.doc_type.is_empty(),
            "'type' field should exists and has at least one entry for the required 'ref' metadata definition"
        );

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

        Ok(Self::Specified {
            allowed_type: exp_ref_types,
            multiple: spec.multiple,
            optional,
        })
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
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
                );
            } else if !optional {
                doc.report()
                    .missing_field("ref", &format!("{context}, document must have ref field"));
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self
            && let Some(doc_ref) = doc.doc_meta().doc_ref()
        {
            doc.report().unknown_field(
                "ref",
                &doc_ref.to_string(),
                &format!("{context}, document does not expect to have a ref field"),
            );
            return Ok(false);
        }

        Ok(true)
    }
}

/// Validate all the document references by the defined validation rules,
/// plus conducting additional validations with the provided `validator`.
/// Document all possible error in doc report (no fail fast)
// TODO: Change the return type to `Result<()>` when other rules are updated.
pub(crate) fn doc_refs_check<Validator>(
    doc_refs: &DocumentRefs,
    exp_ref_types: &[DocType],
    multiple: bool,
    field_name: &str,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
    validator: Validator,
) -> anyhow::Result<bool>
where
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
        if let Some(ref ref_doc) = provider.try_get_doc(dr)? {
            let is_valid = referenced_doc_type_check(ref_doc, exp_ref_types, field_name, report)
                && referenced_doc_ref_check(ref_doc, dr, field_name, report)
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
fn referenced_doc_ref_check(
    ref_doc: &CatalystSignedDocument,
    original_doc_ref: &DocumentRef,
    field_name: &str,
    report: &ProblemReport,
) -> bool {
    let context = &format!("Referenced document validation for the `{field_name}` field");
    let Ok(doc_ref) = ref_doc.doc_ref() else {
        report.functional_validation(
            "Cannot calculate a document reference of the fetched document",
            context,
        );
        return false;
    };

    if &doc_ref != original_doc_ref {
        report.invalid_value(
            "document reference",
            &format!("{doc_ref}",),
            &format!("{original_doc_ref}",),
            context,
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
            &exp_ref_types.iter().map(ToString::to_string).join(","),
            &format!("Invalid referenced document type, during validation of {field_name} field"),
        );
        return false;
    }
    true
}
