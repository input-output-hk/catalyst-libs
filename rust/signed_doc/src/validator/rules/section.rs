//! `section` rule type impl.

use crate::CatalystSignedDocument;

/// `section` field validation rule
pub(crate) struct SectionRule {
    /// optional flag for the `section` field
    pub(crate) optional: bool,
}

impl SectionRule {
    /// Field validation rule
    pub(crate) async fn check(&self, doc: &CatalystSignedDocument) -> anyhow::Result<bool> {
        if doc.doc_meta().section().is_none() && !self.optional {
            doc.report()
                .missing_field("section", "Document must have a section field");
            return Ok(false);
        }
        Ok(true)
    }
}
