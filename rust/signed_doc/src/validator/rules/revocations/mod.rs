//! `revocations` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_signed_doc_spec::{is_required::IsRequired, metadata::revocations::Revocations};

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// `revocations` field validation rule
#[derive(Debug)]
pub(crate) enum RevocationsRule {
    /// Is 'revocations' specified
    Specified {
        /// optional flag for the `revocations` field
        optional: bool,
    },
    /// 'revocations' is not specified
    NotSpecified,
}

impl CatalystSignedDocumentValidationRule for RevocationsRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl RevocationsRule {
    /// Constructs a rule from the given spec.
    pub(crate) fn new(spec: &Revocations) -> Self {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                return Self::NotSpecified;
            },
        };

        Self::Specified { optional }
    }

    /// Performs the field validation.
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        if let Self::Specified { optional } = self
            && doc.doc_meta().revocations().is_none()
            && !optional
        {
            doc.report().missing_field(
                "revocations",
                "Document must have 'revocations' field specified",
            );
            return false;
        }
        if let Self::NotSpecified = self
            && doc.doc_meta().revocations().is_some()
        {
            doc.report().unknown_field(
                "revocations",
                &format!(
                    "{:#?}",
                    doc.doc_meta().revocations().map(ToString::to_string)
                ),
                "Document does not expect to have a 'revocations' field",
            );
            return false;
        }

        true
    }
}
