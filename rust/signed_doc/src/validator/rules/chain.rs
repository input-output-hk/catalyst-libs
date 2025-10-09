//! `chain` rule type impl.

use std::collections::HashMap;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocumentRef};

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
                            &format!(
                                "Cannot find the Chained Document ({}) from the provider",
                                current_key
                            ),
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };
                    let Some(chained_doc) = signed_docs.get(&chained_key) else {
                        doc.report().other(
                            &format!(
                                "Cannot find the Chained Document ({}) from the provider",
                                chained_key
                            ),
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

                // validate remainings in the pool
                let remaining_refs: Vec<_> = signed_docs
                    .keys()
                    .filter(|k| !visited.contains(k))
                    .cloned()
                    .collect();

                for remaining_ref in remaining_refs {
                    let Some(doc) = signed_docs.get(&remaining_ref) else {
                        doc.report().other(
                            &format!(
                                "Cannot find the Chained Document ({}) from the provider",
                                remaining_ref
                            ),
                            "Chained Documents validation",
                        );
                        return Ok(false);
                    };

                    let chained_ref = doc
                        .doc_meta()
                        .chain()
                        .map(|chain| chain.document_ref())
                        .flatten();
                    if chained_ref.is_some_and(|doc_ref| visited.contains(&doc_ref)) {
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

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
        DocType,
    };

    #[tokio::test]
    async fn test_without_chaining_documents() {
        let doc_type = UuidV4::new();
        let doc_id = UuidV7::new();
        let doc_ver = UuidV7::new();

        let provider = TestCatalystProvider::default();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(doc_ver))
            .build();

        let rule = ChainRule::NotSpecified;
        assert!(rule.check(&doc, &provider).await.unwrap());
        let rule = ChainRule::Specified { optional: true };
        assert!(rule.check(&doc, &provider).await.unwrap());
        let rule = ChainRule::Specified { optional: false };
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => true;
        "valid minimal chained documents (0, -1)"
    )]
    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => true;
        "valid long chained documents (0, 1, 2, 3, -4)"
    )]
    #[tokio::test]
    async fn test_valid_chained_documents(
        (provider, doc): (TestCatalystProvider, CatalystSignedDocument)
    ) -> bool {
        let rule = ChainRule::Specified { optional: false };

        rule.check(&doc, &provider).await.unwrap()
    }

    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => false;
        "missing collaborators field"
    )]
    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => false;
        "not have the same id as the document being chained to"
    )]
    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => false;
        "not have a ver that is greater than the ver being chained to"
    )]
    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => false;
        "not the same type as the chained document"
    )]
    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => false;
        "chaining to the document already chained to by another document"
    )]
    #[test_case(
        {
            let provider = TestCatalystProvider::default();
            let doc = Builder::new().build();

            (provider, doc)
        } => false;
        "not have its absolute height exactly one more than the height of the document being chained to"
    )]
    #[tokio::test]
    async fn test_invalid_chained_documents(
        (provider, doc): (TestCatalystProvider, CatalystSignedDocument)
    ) -> bool {
        let rule = ChainRule::Specified { optional: false };

        rule.check(&doc, &provider).await.unwrap()
    }
}
