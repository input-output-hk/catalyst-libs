//! Original Author Validation Rule

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use anyhow::ensure;
use catalyst_signed_doc_spec::signers::update::Update;
use catalyst_types::catalyst_id::CatalystId;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Context for the validation problem report.
const REPORT_CONTEXT: &str = "Document ownership validation";

/// Document Ownership Validation Rule
#[derive(Debug)]
pub(crate) struct DocumentOwnershipRule {
    /// Collaborators are allowed.
    allow_collaborators: bool,
}

impl DocumentOwnershipRule {
    /// Creates `DocumentOwnershipRule` from specs.
    pub(crate) fn new(spec: &Update) -> anyhow::Result<Self> {
        ensure!(spec.author, "'author' field must always be equal to `true`");

        Ok(Self {
            allow_collaborators: spec.collaborators,
        })
    }

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

        if doc_id == doc.doc_ver()? && doc.authors().len() != 1 {
            doc.report().functional_validation(
                "Document must only be signed by one author",
                REPORT_CONTEXT,
            );
            return Ok(false);
        }

        if let Some(first_doc) = provider.try_get_first_doc(doc_id).await? {
            if self.allow_collaborators {
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
                    .collect::<HashSet<CatalystId>>();
                allowed_authors.extend(
                    last_doc
                        .doc_meta()
                        .collaborators()
                        .iter()
                        .map(CatalystId::as_short_id),
                );
                let doc_authors = doc.authors().into_iter().collect::<HashSet<_>>();

                // all elements of the `doc_authors` should be intersecting with the
                // `allowed_authors`
                let is_valid =
                    allowed_authors.intersection(&doc_authors).count() == doc_authors.len();

                if !is_valid {
                    doc.report().functional_validation(
                         &format!(
                            "Document must only be signed by original author and/or by collaborators defined in the previous version. Allowed signers: {:?}, Document signers: {:?}",
                            allowed_authors.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                            doc_authors.iter().map(|v| v.to_string()).collect::<Vec<_>>()
                        ),
                        REPORT_CONTEXT,
                    );
                }
                return Ok(is_valid);
            } else {
                // No collaborators are allowed
                let is_valid = first_doc.authors() == doc.authors();
                if !is_valid {
                    doc.report().functional_validation(
                        "Document authors must match the author from the first version",
                        REPORT_CONTEXT,
                    );
                }
                return Ok(is_valid);
            }
        }

        Ok(true)
    }
}
