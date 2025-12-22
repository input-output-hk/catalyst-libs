//! Catalyst Signed Documents validation logic

pub(crate) mod rules;

#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
use std::fmt::Debug;

#[cfg(not(target_arch = "wasm32"))]
use dashmap::DashMap;

use crate::{
    CatalystSignedDocument, metadata::DocType, providers::Provider,
    validator::rules::documents_rules_from_spec,
};

/// `CatalystSignedDocument` validation rule trait
#[cfg(not(target_arch = "wasm32"))]
pub trait CatalystSignedDocumentValidationRule: 'static + Send + Sync + Debug {
    /// Validates `CatalystSignedDocument`, return `false` if the provided
    /// `CatalystSignedDocument` violates some validation rules with properly filling the
    /// problem report.
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool>;
}

/// `CatalystSignedDocument` validation rule trait
#[cfg(target_arch = "wasm32")]
pub trait CatalystSignedDocumentValidationRule: Debug {
    /// Validates `CatalystSignedDocument`, return `false` if the provided
    /// `CatalystSignedDocument` violates some validation rules with properly filling the
    /// problem report.
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool>;
}

/// Struct represented a collection of rules
pub type Rules = Vec<Box<dyn CatalystSignedDocumentValidationRule>>;

/// `CatalystSignedDocument` validator type.
#[cfg(not(target_arch = "wasm32"))]
pub struct Validator(DashMap<DocType, Rules>);

/// `CatalystSignedDocument` validator type.
#[cfg(target_arch = "wasm32")]
pub struct Validator(HashMap<DocType, Rules>);

impl Validator {
    /// # Panics
    /// - Cannot fail to initialize validation rules. Should never happen.
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        Self(
            documents_rules_from_spec()
                .expect("Cannot fail to initialize validation rules. Should never happen.")
                .collect(),
        )
    }

    /// A comprehensive document type based validation of the `CatalystSignedDocument`.
    /// Includes time based validation of the `id` and `ver` fields based on the provided
    /// `future_threshold` and `past_threshold` threshold values (in seconds).
    /// Return true if it is valid, otherwise return false.
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    pub fn validate(
        &self,
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

        let Some(rules) = self.0.get(doc_type) else {
            doc.report().invalid_value(
                "`type`",
                &doc.doc_type()?.to_string(),
                "Must be a known document type value",
                "Unsupported document type",
            );
            return Ok(false);
        };

        let res = rules
            .iter()
            .map(|v| v.check(doc, provider))
            .collect::<anyhow::Result<Vec<_>>>()?
            .iter()
            .all(|res| *res);
        Ok(res)
    }

    /// Extend the current defined validation rules set for the provided document type.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn extend_rules_per_document(
        &mut self,
        doc_type: DocType,
        rule: impl CatalystSignedDocumentValidationRule,
    ) {
        self.0.entry(doc_type).or_default().push(Box::new(rule));
    }

    /// Extend the current defined validation rules set for the provided document type.
    #[cfg(target_arch = "wasm32")]
    pub fn extend_rules_per_document(
        &self,
        doc_type: DocType,
        rule: impl CatalystSignedDocumentValidationRule,
    ) {
        self.0.entry(doc_type).or_default().push(Box::new(rule));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_rules_init_test() {
        let _unused = Validator::new();
    }
}
