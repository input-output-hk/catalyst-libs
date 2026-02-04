//! `chain` rule type impl.

use anyhow::ensure;
use catalyst_signed_doc_spec::{
    is_required::IsRequired,
    metadata::{chain::Chain as ChainSpec, collaborators::Collaborators},
};

use crate::{
    CatalystSignedDocument, Chain,
    providers::{CatalystSignedDocumentProvider, Provider},
    validator::{CatalystSignedDocumentValidationRule, rules::doc_ref::doc_refs_check},
};

#[cfg(test)]
mod tests;

/// `chain` field validation rule
#[derive(Debug)]
pub(crate) enum ChainRule {
    /// Is 'chain' specified
    Specified {
        /// optional flag for the `chain` field
        optional: bool,
    },
    /// 'chain' is not specified
    NotSpecified,
}

impl CatalystSignedDocumentValidationRule for ChainRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider)?;
        Ok(!doc.report().is_problematic())
    }
}

impl ChainRule {
    /// Generating `ChainRule` from specs
    pub(crate) fn new(
        spec: &ChainSpec,
        collaborators_spec: &Collaborators,
    ) -> anyhow::Result<Self> {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                return Ok(Self::NotSpecified);
            },
        };

        ensure!(
            matches!(collaborators_spec.required, IsRequired::Excluded),
            "Chained Documents do not support collaborators"
        );

        Ok(Self::Specified { optional })
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<()> {
        let chain = doc.doc_meta().chain();

        if let Self::Specified { optional } = self {
            if chain.is_none() && !optional {
                doc.report()
                    .missing_field("chain", "Document must have 'chain' field");
                return Ok(());
            }

            // perform integrity validation
            if let Some(doc_chain) = chain {
                Self::chain_check(doc_chain, doc, provider)?;
            }
        }
        if let Self::NotSpecified = self
            && chain.is_some()
        {
            doc.report().unknown_field(
                "chain",
                &doc.doc_meta()
                    .chain()
                    .iter()
                    .map(ToString::to_string)
                    .reduce(|a, b| format!("{a}, {b}"))
                    .unwrap_or_default(),
                "Document does not expect to have 'chain' field",
            );
        }

        Ok(())
    }

    /// `chain` metadata field checks
    fn chain_check(
        doc_chain: &Chain,
        doc: &CatalystSignedDocument,
        provider: &dyn CatalystSignedDocumentProvider,
    ) -> anyhow::Result<()> {
        const CONTEXT: &str = "Chained Documents validation";

        if doc_chain.document_ref().is_none() && doc_chain.height() != 0 {
            doc.report().functional_validation(
                "The chain height must be zero when there is no chained doc",
                CONTEXT,
            );
            return Ok(());
        }
        if doc_chain.height() == 0 && doc_chain.document_ref().is_some() {
            doc.report().functional_validation(
                "The next Chained Document must not exist while the height is zero",
                CONTEXT,
            );
            return Ok(());
        }

        if let Some(chained_ref) = doc_chain.document_ref() {
            let Ok(expected_doc_type) = doc.doc_type() else {
                doc.report().missing_field("type", CONTEXT);
                return Ok(());
            };

            let chain_validator = |chained_doc: &CatalystSignedDocument| {
                let Ok(doc_id) = doc.doc_id() else {
                    doc.report()
                        .missing_field("id", "Missing id field in the document");
                    return false;
                };
                let Ok(chained_id) = chained_doc.doc_id() else {
                    doc.report()
                        .missing_field("id", "Missing id field in the chained document");
                    return false;
                };
                // have the same id as the document being chained to.
                if chained_id != doc_id {
                    doc.report().functional_validation(
                        "Must have the same id as the document being chained to",
                        CONTEXT,
                    );
                    return false;
                }

                let Ok(doc_ver) = doc.doc_ver() else {
                    doc.report()
                        .missing_field("ver", "Missing ver field in the document");
                    return false;
                };
                let Ok(chained_ver) = chained_doc.doc_ver() else {
                    doc.report()
                        .missing_field("ver", "Missing ver field in the chained document");
                    return false;
                };
                // have a ver that is greater than the ver being chained to.
                if chained_ver > doc_ver {
                    doc.report().functional_validation(
                        "Must have a ver that is greater than the ver being chained to",
                        CONTEXT,
                    );
                    return false;
                }

                let Some(chained_height) = chained_doc.doc_meta().chain().map(crate::Chain::height)
                else {
                    doc.report()
                        .missing_field("chain", "Missing chain field in the chained document");
                    return false;
                };

                // chain doc must not be negative
                if chained_height < 0 {
                    doc.report().functional_validation(
                        "The height of the document being chained to must be positive number",
                        CONTEXT,
                    );
                    return false;
                }

                // have its absolute height exactly one more than the height of the
                // document being chained to.
                if !matches!(
                    i32::abs(doc_chain.height()).checked_sub(i32::abs(chained_height)),
                    Some(1)
                ) {
                    doc.report().functional_validation(
                                "Must have its absolute height exactly one more than the height of the document being chained to",
                                CONTEXT,
                            );
                    return false;
                }

                true
            };

            doc_refs_check(
                &vec![chained_ref.clone()].into(),
                std::slice::from_ref(expected_doc_type),
                false,
                "chain",
                provider,
                doc.report(),
                chain_validator,
            )?;
        }

        Ok(())
    }
}
