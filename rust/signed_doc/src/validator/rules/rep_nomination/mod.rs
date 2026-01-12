//! `rep_domination` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{is_required::IsRequired};

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// `rep_domination` field validation rule
#[derive(Debug)]
pub(crate) struct RepNominationRule;

impl CatalystSignedDocumentValidationRule for RepNominationRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl RepNominationRule {
    /// Generating `RepNominationRule` from specs
    pub(crate) fn new<Provider>(
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<Self> {
        /* if doc.report().is_problematic() {
            anyhow::bail!("Provided document is not valid {:?}", doc.report())
        }
        anyhow::ensure!(
            doc.doc_type()? == &REP_NOMINATION,
            "Document must be Contest Ballot type"
        ); */

        todo!();
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        todo!();
    }
}
