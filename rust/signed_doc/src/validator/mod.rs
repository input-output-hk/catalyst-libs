//! Catalyst Signed Documents validation logic

pub(crate) mod rules;

use std::{fmt::Debug, sync::LazyLock};

use dashmap::DashMap;
use futures::{StreamExt, TryStreamExt};

use crate::{
    CatalystSignedDocument, metadata::DocType, providers::Provider,
    validator::rules::documents_rules_from_spec,
};

/// `CatalystSignedDocument` validation rule trait
#[async_trait::async_trait]
pub trait CatalystSignedDocumentValidationRule: 'static + Send + Sync + Debug {
    /// Validates `CatalystSignedDocument`, return `false` if the provided
    /// `CatalystSignedDocument` violates some validation rules with properly filling the
    /// problem report.
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool>;
}

/// Struct represented a collection of rules
pub(crate) type Rules = Vec<Box<dyn CatalystSignedDocumentValidationRule>>;

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<DashMap<DocType, Rules>> = LazyLock::new(document_rules_init);

/// `DOCUMENT_RULES` initialization function
#[allow(clippy::expect_used)]
fn document_rules_init() -> DashMap<DocType, Rules> {
    let document_rules_map: DashMap<DocType, Rules> = documents_rules_from_spec()
        .expect("cannot fail to initialize validation rules")
        .collect();

    document_rules_map
}

/// A comprehensive document type based validation of the `CatalystSignedDocument`.
/// Includes time based validation of the `id` and `ver` fields based on the provided
/// `future_threshold` and `past_threshold` threshold values (in seconds).
/// Return true if it is valid, otherwise return false.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate(
    doc: &CatalystSignedDocument,
    provider: &impl Provider,
) -> anyhow::Result<bool> {
    let Ok(doc_type) = doc.doc_type() else {
        doc.report().missing_field(
            "type",
            "Can't get a document type during the validation process",
        );
        return Ok(false);
    };

    let Some(rules) = DOCUMENT_RULES.get(doc_type) else {
        doc.report().invalid_value(
            "`type`",
            &doc.doc_type()?.to_string(),
            "Must be a known document type value",
            "Unsupported document type",
        );
        return Ok(false);
    };

    let iter = rules.iter().map(|v| v.check(doc, provider));
    let res = futures::stream::iter(iter)
        .buffer_unordered(rules.len())
        .try_collect::<Vec<_>>()
        .await?
        .iter()
        .all(|res| *res);
    Ok(res)
}

/// Extend the current defined validation rules set for the provided document type.
pub fn extend_rules_per_document(
    doc_type: DocType,
    rule: impl CatalystSignedDocumentValidationRule,
) {
    DOCUMENT_RULES
        .entry(doc_type)
        .or_default()
        .push(Box::new(rule));
}

#[cfg(test)]
mod tests {
    use crate::validator::document_rules_init;

    #[test]
    fn document_rules_init_test() {
        document_rules_init();
    }
}
