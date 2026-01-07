//! `revocations` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{
    DocSpecs, DocumentName, is_required::IsRequired, metadata::reply::Reply,
};

use crate::{
    CatalystSignedDocument, DocType,
    providers::Provider,
    validator::{CatalystSignedDocumentValidationRule, rules::doc_ref::doc_refs_check},
};

/// `revocations` field validation rule
#[derive(Debug)]
pub(crate) enum RevocationsRule {
    /// Is 'revocations' specified
    Specified {
        /// optional flag for the `revocations` field
        optional: bool,
    },
    /// 'revocations' is not specified
    NotSpecified,
}

impl CatalystSignedDocumentValidationRule for RevocationsRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider)
    }
}

impl RevocationsRule {
    pub(crate) fn new(
        docs: &DocSpecs,
        spec: &Reply,
    ) -> anyhow::Result<Self> {
        // TODO:
        unimplemented!()
    }

    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        // TODO:
        unimplemented!()
    }
}