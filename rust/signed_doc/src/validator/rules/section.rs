//! `section` rule type impl.

use crate::CatalystSignedDocument;

/// `section` field validation rule
pub(crate) enum SectionRule {
    /// Is 'section' specified
    Specified {
        /// optional flag for the `section` field
        optional: bool,
    },
    /// 'section' is not specified
    NotSpecified,
}

impl SectionRule {
    /// Field validation rule
    pub(crate) async fn check(&self, doc: &CatalystSignedDocument) -> anyhow::Result<bool> {
        if let Self::Specified { optional } = self {
            if doc.doc_meta().section().is_none() && !optional {
                doc.report()
                    .missing_field("section", "Document must have a section field");
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if let Some(section) = doc.doc_meta().section() {
                doc.report().unknown_field(
                    "section",
                    &section.to_string(),
                    "Document does not expect to have a section field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
