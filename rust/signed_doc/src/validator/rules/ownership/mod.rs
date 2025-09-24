//! Original Author Validation Rule

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use catalyst_types::catalyst_id::CatalystId;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Context for the validation problem report.
const REPORT_CONTEXT: &str = "Document ownership validation";

/// Returns `true` if the document has a single author.
///
/// If not, it adds to the document's problem report.
fn single_author_check(doc: &CatalystSignedDocument) -> bool {
    let is_valid = doc.authors().len() == 1;
    if !is_valid {
        doc.report()
            .functional_validation("Document must only be signed by one author", REPORT_CONTEXT);
    }
    is_valid
}

/// Document Ownership Validation Rule
#[derive(Debug)]
pub(crate) struct DocumentOwnershipRule {
    /// Collaborators are allowed.
    pub(crate) allow_collaborators: bool,
}

impl DocumentOwnershipRule {
    /// Check document ownership rule
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let doc_id = doc.doc_id()?;
        let first_doc_opt = provider.try_get_first_doc(doc_id).await?;

        if self.allow_collaborators {
            if let Some(first_doc) = first_doc_opt {
                // This a new version of an existing `doc_id`
                let Some(last_doc) = provider.try_get_last_doc(doc_id).await? else {
                    anyhow::bail!(
                        "A latest version of the document must exist if a first version exists"
                    );
                };

                // Create sets of authors for comparison, ensure that they are in the same form
                // (e.g. each `kid` is in `URI form`).
                //
                // Allowed authors for this document are the original author, and collaborators
                // defined in the last published version of the Document ID.
                let mut allowed_authors = first_doc
                    .authors()
                    .into_iter()
                    .map(CatalystId::as_uri)
                    .collect::<HashSet<CatalystId>>();
                allowed_authors.extend(
                    last_doc
                        .doc_meta()
                        .collaborators()
                        .iter()
                        .cloned()
                        .map(CatalystId::as_uri),
                );
                let doc_authors = doc
                    .authors()
                    .into_iter()
                    .map(CatalystId::as_uri)
                    .collect::<HashSet<_>>();

                let is_valid = allowed_authors.intersection(&doc_authors).count() > 0;

                if !is_valid {
                    doc.report().functional_validation(
                        "Document must only be signed by original author and/or by collaborators defined in the previous version",
                        REPORT_CONTEXT,
                    );
                }
                return Ok(is_valid);
            }

            // This is a first version of the doc
            return Ok(single_author_check(doc));
        }

        // No collaborators are allowed
        if let Some(first_doc) = first_doc_opt {
            // This a new version of an existing `doc_id`
            let is_valid = first_doc.authors() == doc.authors();
            if !is_valid {
                doc.report().functional_validation(
                    "Document authors must match the author from the first version",
                    REPORT_CONTEXT,
                );
            }
            return Ok(is_valid);
        }

        // This is a first version of the doc
        Ok(single_author_check(doc))
    }
}
