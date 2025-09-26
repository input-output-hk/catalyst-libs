//! `chain` rule type impl.

use std::collections::HashMap;

use crate::{
    providers::{CatalystIdProvider, CatalystSignedDocumentProvider},
    CatalystSignedDocument,
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
                        tmp.push(((doc.doc_id()?, doc.doc_ver()?), doc));
                    }
                    tmp.into_iter().collect()
                };

                if let Some(chained_ref) = chain.document_ref() {
                    let chained_key = (chained_ref.id().clone(), chained_ref.ver().clone());
                    let Some(chained_doc) = signed_docs.get(&chained_key) else {
                        return Ok(false);
                    };

                    // not have collaborators.
                    if !chained_doc.doc_meta().collaborators().is_empty() {
                        return Ok(false);
                    }

                    // have the same id as the document being chained to.
                    if chained_doc.doc_id()? != doc.doc_id()? {
                        return Ok(false);
                    }

                    // have a ver that is greater than the ver being chained to.
                    if chained_doc.doc_ver()? > doc.doc_ver()? {
                        return Ok(false);
                    }

                    // have the same type as the chained document.
                    if chained_doc.doc_type()? != doc.doc_type()? {
                        return Ok(false);
                    }

                    // have parameters match.
                    if chained_doc.doc_meta().parameters() != doc.doc_meta().parameters() {
                        return Ok(false);
                    }

                    // have not be chaining to a document already chained to by another
                    // document.

                    // have its absolute height exactly one more than the height of the
                    // document being chained to.
                } else {
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
