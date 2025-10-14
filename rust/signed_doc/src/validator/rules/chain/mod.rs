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
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn new(
        _docs: &DocSpecs,
        spec: &Chain,
    ) -> anyhow::Result<Self> {
        let optional = match spec.required {
            IsRequired::Yes => false,
            IsRequired::Optional => true,
            IsRequired::Excluded => {
                return Ok(Self::NotSpecified);
            },
        };

        Ok(Self::Specified { optional })
    }

    /// Field validation rule
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::unused_async)]
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
            if let Some(chain) = chain {
                if let Some(chained_ref) = chain.document_ref() {
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
                        return Ok(false);
                    }

                    // have a ver that is greater than the ver being chained to.
                    if chained_doc.doc_ver()? > doc.doc_ver()? {
                        doc.report().functional_validation(
                            "Must have a ver that is greater than the ver being chained to",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have the same type as the chained document.
                    if chained_doc.doc_type()? != doc.doc_type()? {
                        doc.report().functional_validation(
                            "Must have the same type as the chained document",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have parameters match.
                    if chained_doc.doc_meta().parameters() != doc.doc_meta().parameters() {
                        doc.report().functional_validation(
                            "Must have parameters match",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have its absolute height exactly one more than the height of the
                    // document being chained to.
                    let current_height = doc.doc_meta().chain().map_or(0, crate::Chain::height);
                    let chained_height = chained_doc
                        .doc_meta()
                        .chain()
                        .map_or(0, crate::Chain::height);

                    if !matches!(
                        i32::abs(current_height).checked_sub(i32::abs(chained_height)),
                        Some(1)
                    ) {
                        doc.report().functional_validation(
                            "Must have its absolute height exactly one more than the height of the document being chained to",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }
                }
            }
        }
        if let Self::NotSpecified = self {
            if chain.is_some() {
                doc.report().unknown_field(
                    "chain",
                    &format!(
                        "{:#?}",
                        doc.doc_meta()
                            .chain()
                            .iter()
                            .map(ToString::to_string)
                            .reduce(|a, b| format!("{a}, {b}"))
                    ),
                    "Document does not expect to have 'chain' field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
