//! `collaborators` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{is_required::IsRequired, metadata::collaborators::Collaborators};

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// `collaborators` field validation rule
#[derive(Debug)]
pub(crate) enum CollaboratorsRule {
    /// Is 'collaborators' specified
    Specified {
        /// optional flag for the `collaborators` field
        optional: bool,
    },
    /// 'collaborators' is not specified
    NotSpecified,
}

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for CollaboratorsRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl CollaboratorsRule {
    /// Generating `CollaboratorsRule` from specs
    pub(crate) fn new(spec: &Collaborators) -> Self {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                return Self::NotSpecified;
            },
        };

        Self::Specified { optional }
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        if let Self::Specified { optional } = self
            && doc.doc_meta().collaborators().is_empty()
            && !optional
        {
            doc.report().missing_field(
                "collaborators",
                "Document must have at least one entry in 'collaborators' field",
            );
            return false;
        }
        if let Self::NotSpecified = self
            && !doc.doc_meta().collaborators().is_empty()
        {
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
            return false;
        }

        true
    }
}
