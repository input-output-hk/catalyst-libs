//! `chain` rule type impl.

use std::collections::HashMap;

use crate::{
    providers::{CatalystIdProvider, CatalystSignedDocumentProvider},
    CatalystSignedDocument, DocumentRef,
};

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
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider + CatalystIdProvider,
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
                let mut visited: Vec<DocumentRef> = Vec::with_capacity(signed_docs.len());

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

                    visited.push(chained_key.clone());

                    let Some(current_doc) = signed_docs.get(&current_key) else {
                        doc.report().other(
                            "Cannot find the Chained Document from the provider",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };
                    let Some(chained_doc) = signed_docs.get(&chained_key) else {
                        doc.report().other(
                            "Cannot find the Chained Document from the provider",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };

                    // not have collaborators.
                    if !chained_doc.doc_meta().collaborators().is_empty() {
                        doc.report().invalid_value(
                            "collaborators",
                            &format!("{} entries", chained_doc.doc_meta().collaborators().len()),
                            "Must not have collaborators",
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    }

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
                    let current_height = current_doc.doc_meta().chain().map(|chain| chain.height());
                    let chained_height = chained_doc.doc_meta().chain().map(|chain| chain.height());

                    if let (Some(current_height), Some(chained_height)) =
                        (current_height, chained_height)
                    {
                        if i32::abs(current_height) - i32::abs(chained_height) != 1 {
                            doc.report().functional_validation(
                                "Must have parameters match",
                                "Chained Documents validation",
                            );
                            return Ok(false);
                        }
                    }

                    current_chaining_ref = Some(DocumentRef::try_from(chained_doc)?);
                    visiting_chained_ref = chained_doc
                        .doc_meta()
                        .chain()
                        .map(|v| v.document_ref())
                        .flatten()
                        .cloned();

                    // incomplete chain
                    if visiting_chained_ref.is_none()
                        && current_doc
                            .doc_meta()
                            .chain()
                            .is_some_and(|chain| chain.height() != 0)
                    {
                        return Ok(false);
                    }
                    if current_doc
                        .doc_meta()
                        .chain()
                        .is_some_and(|chain| chain.height() == 0)
                        && visiting_chained_ref.is_some()
                    {
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
