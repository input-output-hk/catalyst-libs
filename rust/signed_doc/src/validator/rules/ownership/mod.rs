//! Original Author Validation Rule

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use anyhow::ensure;
use catalyst_signed_doc_spec::{
    is_required::IsRequired,
    signers::update::{Collaborators, Update},
    DocSpec,
};
use catalyst_types::catalyst_id::CatalystId;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Context for the validation problem report.
const REPORT_CONTEXT: &str = "Document ownership validation";

/// Document Ownership Validation Rule
#[derive(Debug)]
pub(crate) enum DocumentOwnershipRule {
    /// Collaborators are allowed, based on the 'collaborators' metadata field.
    CollaboratorsFieldBased,
    /// Collaborators are allowed, based on the 'ref' metadata field.
    RefFieldBased,
    /// Collaborators are not allowed, only original author.
    WitoutCollaborators,
}

impl DocumentOwnershipRule {
    /// Creates `DocumentOwnershipRule` from specs.
    pub(crate) fn new(
        spec: &Update,
        doc_spec: &DocSpec,
    ) -> anyhow::Result<Self> {
        ensure!(spec.author, "'author' field must always be equal to `true`");

        match spec.collaborators {
            Collaborators::Collaborators => {
                ensure!(
                    doc_spec.metadata.collaborators.required != IsRequired::Excluded,
                    "'collaborators' metadata field cannot be 'excluded' if 'update'->'collaborators' is 'collaborators' based"
                );
                Ok(Self::CollaboratorsFieldBased)
            },
            Collaborators::Ref => {
                ensure!(
                    doc_spec.metadata.doc_ref.required == IsRequired::Yes,
                    "'ref' metadata field cannot be 'excluded' or 'optional' if 'update'->'collaborators' is 'ref' based"
                );
                ensure!(
                    doc_spec.metadata.doc_ref.multiple,
                    "'ref' metadata field cannot has multiple document references if 'update'->'collaborators' is 'ref' based"
                );
                Ok(Self::RefFieldBased)
            },
            Collaborators::Excluded => Ok(Self::WitoutCollaborators),
        }
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

        let mut allowed_authors = HashSet::new();
        if let DocumentOwnershipRule::RefFieldBased = self {
            let Some(doc_ref) = doc.doc_meta().doc_ref() else {
                doc.report().missing_field("ref", REPORT_CONTEXT);
                return Ok(false);
            };
            let &[ref doc_ref] = doc_ref.as_slice() else {
                doc.report()
                    .other("'ref' field cannot have multiple values", REPORT_CONTEXT);
                return Ok(false);
            };
            let Some(first_ref_doc) = provider.try_get_first_doc(*doc_ref.id()).await? else {
                doc.report().other(
                    "Cannot find a first version of the referenced document",
                    REPORT_CONTEXT,
                );
                return Ok(false);
            };
            allowed_authors.extend(first_ref_doc.authors());

            let last_doc =
                provider
                    .try_get_last_doc(*doc_ref.id())
                    .await?
                    .ok_or(anyhow::anyhow!(
                        "A latest version of the document must exist if a first version exists"
                    ))?;

            allowed_authors.extend(
                last_doc
                    .doc_meta()
                    .collaborators()
                    .iter()
                    .map(CatalystId::as_short_id),
            );
        } else if let Some(first_doc) = provider.try_get_first_doc(doc_id).await? {
            allowed_authors.extend(first_doc.authors());

            if let DocumentOwnershipRule::CollaboratorsFieldBased = self {
                // This a new version of an existing `doc_id`
                let last_doc = provider
                    .try_get_last_doc(doc_id)
                    .await?
                    .ok_or(anyhow::anyhow!(
                        "A latest version of the document must exist if a first version exists"
                    ))?;

                allowed_authors.extend(
                    last_doc
                        .doc_meta()
                        .collaborators()
                        .iter()
                        .map(CatalystId::as_short_id),
                );
            }
        }

        let doc_authors = doc.authors().into_iter().collect::<HashSet<_>>();

        // all elements of the `doc_authors` should be intersecting with the `allowed_authors` OR
        // `allowed_authors` must be empty
        let is_valid = allowed_authors.is_empty()
            || allowed_authors.intersection(&doc_authors).count() == doc_authors.len();

        if !is_valid {
            doc.report().functional_validation(
                &format!(
                    "Document must only be signed by original author and/or by collaborators defined in the previous version. Allowed signers: {:?}, Document signers: {:?}",
                    allowed_authors.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    doc_authors.iter().map(ToString::to_string).collect::<Vec<_>>()
                ),
                REPORT_CONTEXT
            );
        }
        Ok(is_valid)
    }
}
