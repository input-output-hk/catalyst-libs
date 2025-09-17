//! Original Author Validation Rule

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use catalyst_types::catalyst_id::CatalystId;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Returns `true` if the document has a single author.
///
/// If not, it adds to the document's problem report.
fn single_author_check(doc: &CatalystSignedDocument) -> bool {
    let is_valid = doc.authors().len() == 1;
    if !is_valid {
        doc.report().functional_validation(
            "New document must only be signed by a single author",
            "Valid documents must only be signed by the original author",
        );
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
                // Allowed authors for this document are the original author, and collaborators
                // defined in the last published version of the Document ID.
                let mut allowed_authors = first_doc
                    .authors()
                    .into_iter()
                    .collect::<HashSet<CatalystId>>();
                allowed_authors.extend(last_doc.doc_meta().collaborators().iter().cloned());

                let doc_authors = doc.authors().into_iter().collect::<HashSet<CatalystId>>();

                let is_valid = allowed_authors.intersection(&doc_authors).count() > 0;

                if !is_valid {
                    doc.report().functional_validation(
                        "New document must only be signed by a single author or collaborators defined in the previous version",
                        "Valid documents must only be signed by the original author or known collaborators",
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
                        &format!("New document authors must match the authors from the first version for Document ID {doc_id}"),
                        "Valid documents must only be signed by the original author",
                    );
            }
            return Ok(is_valid);
        }

        // This is a first version of the doc
        Ok(single_author_check(doc))
    }
}
