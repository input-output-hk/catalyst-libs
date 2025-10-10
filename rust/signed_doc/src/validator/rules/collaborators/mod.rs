//! `collaborators` rule type impl.

#[cfg(test)]
mod tests;

use crate::CatalystSignedDocument;

/// `collaborators` field validation rule
#[derive(Debug)]
pub(crate) enum CollaboratorsRule {
    /// Is 'collaborators' specified
    #[allow(dead_code)]
    Specified {
        /// optional flag for the `collaborators` field
        optional: bool,
    },
    /// 'collaborators' is not specified
    NotSpecified,
}

impl CollaboratorsRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        if let Self::Specified { optional } = self {
            if doc.doc_meta().collaborators().is_empty() && !optional {
                doc.report().missing_field(
                    "collaborators",
                    "Document must have at least one entry in 'collaborators' field",
                );
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if !doc.doc_meta().collaborators().is_empty() {
                doc.report().unknown_field(
                    "collaborators",
                    &format!(
                        "{:#?}",
                        doc.doc_meta()
                            .collaborators()
                            .iter()
                            .map(ToString::to_string)
                            .reduce(|a, b| format!("{a}, {b}"))
                    ),
                    "Document does not expect to have a 'collaborators' field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
