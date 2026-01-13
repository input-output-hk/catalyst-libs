//! `rep_domination` rule type impl.

#[cfg(test)]
mod tests;

use crate::{
    CatalystSignedDocument, doc_types, providers::Provider,
    validator::CatalystSignedDocumentValidationRule,
};

/// `rep_domination` field validation rule
#[derive(Debug)]
pub(crate) struct RepNominationRule;

impl CatalystSignedDocumentValidationRule for RepNominationRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc, provider)?)
    }
}

impl RepNominationRule {
    /// Document validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider
    ) -> anyhow::Result<bool> {
        // enabling this rule check only for `REP_NOMINATION` doc
        if doc.doc_type()? != &doc_types::REP_NOMINATION {
            return Ok(true);
        }

        todo!();
    }
}
