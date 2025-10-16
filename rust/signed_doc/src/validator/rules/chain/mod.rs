//! `chain` rule type impl.

use catalyst_signed_doc_spec::{is_required::IsRequired, metadata::chain::Chain, DocSpecs};

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

#[cfg(test)]
mod tests;

/// `chain` field validation rule
#[derive(Debug)]
pub(crate) enum ChainRule {
    /// Is 'chain' specified
    #[allow(dead_code)]
    Specified {
        /// optional flag for the `chain` field
        optional: bool,
    },
    /// 'chain' is not specified
    NotSpecified,
}

impl ChainRule {
    /// Generating `ChainRule` from specs
    pub(crate) fn new(
        _docs: &DocSpecs,
        spec: &Chain,
    ) -> Self {
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
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let chain = doc.doc_meta().chain();

        // TODO: the current implementation is only for the direct chained doc,
        // make it recursively checks the entire chain with the same `id` docs.

        if let Self::Specified { optional } = self {
            if chain.is_none() && !optional {
                doc.report()
                    .missing_field("chain", "Document must have 'chain' field");
                return Ok(false);
            }

            // perform integrity validation
            if let Some(doc_chain) = chain {
                if doc_chain.document_ref().is_none() && doc_chain.height() != 0 {
                    doc.report().functional_validation(
                        "The chain height must be zero when there is no chained doc",
                        "Chained Documents validation",
                    );
                }
                if doc_chain.height() == 0 && doc_chain.document_ref().is_some() {
                    doc.report().functional_validation(
                        "The next Chained Document must not exist while the height is zero",
                        "Chained Documents validation",
                    );
                }

                if let Some(chained_ref) = doc_chain.document_ref() {
                    let Some(chained_doc) = provider.try_get_doc(chained_ref).await? else {
                        doc.report().other(
                            &format!(
                                "Cannot find the Chained Document ({chained_ref}) from the provider"
                            ),
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };

                    // have the same id as the document being chained to.
                    if chained_doc.doc_id()? != doc.doc_id()? {
                        doc.report().functional_validation(
                            "Must have the same id as the document being chained to",
                            "Chained Documents validation",
                        );
                    }

                    // have a ver that is greater than the ver being chained to.
                    if chained_doc.doc_ver()? > doc.doc_ver()? {
                        doc.report().functional_validation(
                            "Must have a ver that is greater than the ver being chained to",
                            "Chained Documents validation",
                        );
                    }

                    // have the same type as the chained document.
                    if chained_doc.doc_type()? != doc.doc_type()? {
                        doc.report().functional_validation(
                            "Must have the same type as the chained document",
                            "Chained Documents validation",
                        );
                    }

                    // have parameters match.
                    if chained_doc.doc_meta().parameters() != doc.doc_meta().parameters() {
                        doc.report().functional_validation(
                            "Must have parameters match",
                            "Chained Documents validation",
                        );
                    }

                    if let Some(chained_height) =
                        chained_doc.doc_meta().chain().map(crate::Chain::height)
                    {
                        // chain doc must not be negative
                        if chained_height < 0 {
                            doc.report().functional_validation(
                                "The height of the document being chained to must be positive number",
                                "Chained Documents validation",
                            );
                        }

                        // have its absolute height exactly one more than the height of the
                        // document being chained to.
                        if !matches!(
                            i32::abs(doc_chain.height()).checked_sub(i32::abs(chained_height)),
                            Some(1)
                        ) {
                            doc.report().functional_validation(
                                "Must have its absolute height exactly one more than the height of the document being chained to",
                                "Chained Documents validation",
                            );
                        }
                    }

                    if doc.report().is_problematic() {
                        return Ok(false);
                    }
                }
            }
        }
        if let Self::NotSpecified = self {
            if chain.is_some() {
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
                return Ok(false);
            }
        }

        Ok(true)
    }
}
