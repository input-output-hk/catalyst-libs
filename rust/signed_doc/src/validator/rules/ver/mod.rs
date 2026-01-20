//! Validator for Signed Document Version

#[cfg(test)]
mod tests;

use anyhow::Context;
use catalyst_types::{problem_report::ProblemReport, uuid::UuidV7};
use chrono::Utc;

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// Signed Document `ver` field validation rule
#[derive(Debug)]
pub(crate) struct VerRule;

impl CatalystSignedDocumentValidationRule for VerRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Self::check_inner(doc, provider)
    }
}

impl VerRule {
    /// Validates document `ver` field according to the following rules:
    /// 1. If `provider.future_threshold()` not `None`, document `ver` cannot be too far
    ///    in the future (`future_threshold` arg) from `Utc::now()` based on the provided
    ///    threshold
    /// 2. If `provider.past_threshold()` not `None`, document `ver` cannot be too far
    ///    behind (`past_threshold` arg) from `Utc::now()` based on the provided threshold
    /// 3. Document `ver` cannot be smaller than document `id` field
    /// 4. IF `ver` does not == `id` then a document with `id` and `ver` being equal
    ///    *MUST* exist
    /// 5. When a document with the same `id` already exists, the new document's `ver`
    ///    must be greater than the latest known submitted version for that `id`
    fn check_inner(
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        let Ok(id) = doc.doc_id() else {
            doc.report().missing_field(
                "id",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };
        let Ok(ver) = doc.doc_ver() else {
            doc.report().missing_field(
                "ver",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };

        let mut is_valid = time_threshold_check(ver, provider, doc.report())?;

        if ver < id {
            doc.report().invalid_value(
                "ver",
                &ver.to_string(),
                "ver < id",
                &format!("Document Version {ver} cannot be smaller than Document ID {id}"),
            );
            is_valid = false;
        } else if let Some(last_doc) = provider.try_get_last_doc(id)? {
            let Ok(last_doc_ver) = last_doc.doc_ver() else {
                doc.report().missing_field(
                    "ver",
                    &format!(
                        "Missing `ver` field in the latest known document, for the the id {id}"
                    ),
                );
                return Ok(false);
            };

            if last_doc_ver >= ver {
                doc.report().functional_validation(
                    &format!("New document ver should be greater that the submitted latest known. New document ver: {ver}, latest known ver: {last_doc_ver}"),
                    &format!("Document's `ver` field should continuously increasing, for the the id {id}"),
                );
                is_valid = false;
            }
        } else if ver != id {
            doc.report().functional_validation(
                &format!("`ver` and `id` are not equal, ver: {ver}, id: {id}. Document with `id` and `ver` being equal MUST exist"),
                "Cannot get a first version document from the provider, document for which `id` and `ver` are equal.",
            );
            is_valid = false;
        }

        Ok(is_valid)
    }
}

/// Time threshold validation check.
/// 1. If `provider.future_threshold()` not `None`, document `ver` cannot be too far in
///    the future (`future_threshold` arg) from `Utc::now()` based on the provided
///    threshold
/// 2. If `provider.past_threshold()` not `None`, document `ver` cannot be too far behind
///    (`past_threshold` arg) from `Utc::now()` based on the provided threshold
fn time_threshold_check(
    ver: UuidV7,
    provider: &dyn Provider,
    report: &ProblemReport,
) -> anyhow::Result<bool> {
    let now = Utc::now();
    let time_delta = ver.time().signed_duration_since(now);

    if let Ok(ver_age) = time_delta.to_std() {
        // `now` is earlier than `ver_time`
        if let Some(future_threshold) = provider.future_threshold()
            && ver_age > future_threshold
        {
            report.invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver < now + future_threshold",
                        &format!("Document 'ver' timestamp {ver} cannot be too far in future (threshold: {future_threshold:?}) from now: {now}"),
                    );
            return Ok(false);
        }
    } else {
        // `ver_time` is earlier than `now`
        let ver_age = time_delta
            .abs()
            .to_std()
            .context("BUG! `id_time` must be earlier than `now` at this place")?;

        if let Some(past_threshold) = provider.past_threshold()
            && ver_age > past_threshold
        {
            report.invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver > now - past_threshold",
                        &format!("Document 'ver' timestamp {ver} cannot be too far behind (threshold: {past_threshold:?}) from now: {now:?}",),
                    );
            return Ok(false);
        }
    }

    Ok(true)
}
