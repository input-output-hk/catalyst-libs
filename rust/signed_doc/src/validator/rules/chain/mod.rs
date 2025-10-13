//! `chain` rule type impl.

use std::collections::{HashMap, HashSet};

use catalyst_signed_doc_spec::{is_required::IsRequired, metadata::chain::Chain, DocSpecs};

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocumentRef};

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

        if let Self::Specified { optional } = self {
            if chain.is_none() && !optional {
                doc.report()
                    .missing_field("chain", "Document must have 'chain' field");
                return Ok(false);
            }

            // perform integrity validation
            if let Some(chain) = chain {
                let signed_docs = provider.try_get_all(doc.doc_id()?).await?;
                let signed_docs: HashMap<_, _> = {
                    let mut tmp = Vec::with_capacity(signed_docs.len());
                    for doc in signed_docs {
                        let doc_ref = DocumentRef::try_from(&doc)?;
                        tmp.push((doc_ref, doc));
                    }
                    tmp.into_iter().collect()
                };
                let mut visited: HashSet<DocumentRef> = HashSet::with_capacity(signed_docs.len());

                let mut current_chaining_ref = Some(DocumentRef::try_from(doc)?);
                let mut visiting_chained_ref = chain.document_ref().cloned();
                while let (Some(chaining_ref), Some(chained_ref)) =
                    (current_chaining_ref, visiting_chained_ref)
                {
                    let current_key = DocumentRef::from_without_locator(&chaining_ref);
                    let chained_key = DocumentRef::from_without_locator(&chained_ref);

                    // have not be chaining to a document already chained to by another
                    // document.
                    if visited.contains(&chained_key) {
                        doc.report().other(
                            "Must not be chaining to a document already chained to by another document",
                            "Chained Documents validation"
                        );
                        return Ok(false);
                    }

                    visited.insert(current_key.clone());
                    visited.insert(chained_key.clone());

                    let Some(current_doc) = signed_docs.get(&current_key) else {
                        doc.report().other(
                            &format!(
                                "Cannot find the Chained Document ({current_key}) from the provider"
                            ),
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };
                    let Some(chained_doc) = signed_docs.get(&chained_key) else {
                        doc.report().other(
                            &format!(
                                "Cannot find the Chained Document ({chained_key}) from the provider"
                            ),
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };

                    // not have collaborators.
                    if !chained_doc.doc_meta().collaborators().is_empty()
                        || !current_doc.doc_meta().collaborators().is_empty()
                    {
                        doc.report().invalid_value(
                            "collaborators",
                            &format!("{} entries", chained_doc.doc_meta().collaborators().len()),
                            "Must not have collaborators",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have the same id as the document being chained to.
                    if chained_doc.doc_id()? != current_doc.doc_id()? {
                        doc.report().functional_validation(
                            "Must have the same id as the document being chained to",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have a ver that is greater than the ver being chained to.
                    if chained_doc.doc_ver()? > current_doc.doc_ver()? {
                        doc.report().functional_validation(
                            "Must have a ver that is greater than the ver being chained to",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have the same type as the chained document.
                    if chained_doc.doc_type()? != current_doc.doc_type()? {
                        doc.report().functional_validation(
                            "Must have the same type as the chained document",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have parameters match.
                    if chained_doc.doc_meta().parameters() != current_doc.doc_meta().parameters() {
                        doc.report().functional_validation(
                            "Must have parameters match",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

                    // have its absolute height exactly one more than the height of the
                    // document being chained to.
                    let current_height = current_doc
                        .doc_meta()
                        .chain()
                        .map_or(0, crate::Chain::height);
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

                    current_chaining_ref = Some(DocumentRef::try_from(chained_doc)?);
                    visiting_chained_ref = chained_doc
                        .doc_meta()
                        .chain()
                        .and_then(|v| v.document_ref())
                        .cloned();
                    let current_doc = chained_doc;

                    // incomplete chain
                    if visiting_chained_ref.is_none()
                        && current_doc
                            .doc_meta()
                            .chain()
                            .is_some_and(|chain| chain.height() != 0)
                    {
                        doc.report().functional_validation(
                            "The next Chained Document must exist while the height is not 0",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }
                    if current_doc
                        .doc_meta()
                        .chain()
                        .is_some_and(|chain| chain.height() == 0)
                        && visiting_chained_ref.is_some()
                    {
                        doc.report().functional_validation(
                            "The next Chained Document must not exist while the height is 0",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }
                }

                // validate remaining docs in the pool
                let remaining_refs: Vec<_> = signed_docs
                    .keys()
                    .filter(|k| !visited.contains(k))
                    .cloned()
                    .collect();

                for remaining_ref in remaining_refs {
                    let Some(doc) = signed_docs.get(&remaining_ref) else {
                        doc.report().other(
                            &format!(
                                "Cannot find the Chained Document ({remaining_ref}) from the provider"
                            ),
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };

                    let chained_ref = doc
                        .doc_meta()
                        .chain()
                        .and_then(|chain| chain.document_ref());
                    if chained_ref.is_some_and(|doc_ref| visited.contains(doc_ref)) {
                        doc.report().other(
                            "Either of the two documents being present invalidates the data in the entire chain",
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
