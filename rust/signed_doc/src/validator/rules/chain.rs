//! `chain` rule type impl.

use std::collections::{HashMap, HashSet};

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
                            "Must have parameters match",
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

#[cfg(test)]
mod tests {
    use catalyst_types::{
        catalyst_id::role_index::RoleId,
        uuid::{UuidV4, UuidV7},
    };
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
        validator::rules::utils::create_dummy_key_pair, Chain, DocType,
    };

    mod helper {
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        use catalyst_types::uuid::UuidV7;
        use uuid::{Timestamp, Uuid};

        pub(super) fn get_now_plus_uuidv7(secs: u64) -> UuidV7 {
            let future_time = SystemTime::now()
                .checked_add(Duration::from_secs(secs))
                .unwrap();
            let duration_since_epoch = future_time.duration_since(UNIX_EPOCH).unwrap();

            let unix_secs = duration_since_epoch.as_secs();
            let nanos = duration_since_epoch.subsec_nanos();

            let ts = Timestamp::from_unix(uuid::NoContext, unix_secs, nanos);
            let uuid = Uuid::new_v7(ts);

            UuidV7::try_from_uuid(uuid).unwrap()
        }
    }

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
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();
            
            let last_doc_ver = helper::get_now_plus_uuidv7(60);
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(-1, Some(first_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
        } => true;
        "valid minimal chained documents (0, -1)"
    )]
    #[test_case(
        {
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();

            let intermediate_doc_ver = helper::get_now_plus_uuidv7(60);
            let intermediate = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(intermediate_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(1, Some(first_doc_ref.clone()))
                ))
                .build();
            let intermediate_doc_ref = DocumentRef::try_from(&intermediate).unwrap();
            
            let last_doc_ver = helper::get_now_plus_uuidv7(120);
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(-2, Some(intermediate_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(intermediate_doc_ref), &intermediate).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
        } => true;
        "valid intermediate chained documents (0, 1, -2)"
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
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();
            
            let last_doc_ver = helper::get_now_plus_uuidv7(60);
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                // collaborators field here
                .with_metadata_field(SupportedField::Collaborators(vec![create_dummy_key_pair(RoleId::Role0).2].into()))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(-1, Some(first_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
        } => false;
        "collaborators field exist"
    )]
    #[test_case(
        {
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();
            // with another doc id
            let doc_id_another = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();
            
            let last_doc_ver = helper::get_now_plus_uuidv7(60);
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id_another))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(-1, Some(first_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
        } => false;
        "not have the same id as the document being chained to"
    )]
    #[test_case(
        {
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();
            
            // same version
            let last_doc_ver = first_doc_ver;
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(-1, Some(first_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
        } => false;
        "not have a ver that is greater than the ver being chained to"
    )]
    #[test_case(
        {
            let doc_type = UuidV4::new();
            // with another doc type
            let doc_type_another = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();
            
            let last_doc_ver = helper::get_now_plus_uuidv7(60);
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type_another)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    Chain::new(-1, Some(first_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
        } => false;
        "not the same type as the chained document"
    )]
    #[test_case(
        {
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();

            let mut doc = None;
            for _ in 0..2 {
                let last_doc_ver = helper::get_now_plus_uuidv7(60);
                let last = Builder::new()
                    .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                    .with_metadata_field(SupportedField::Id(doc_id))
                    .with_metadata_field(SupportedField::Ver(last_doc_ver))
                    .with_metadata_field(SupportedField::Chain(
                        Chain::new(-1, Some(first_doc_ref.clone()))
                    ))
                    .build();
                let last_doc_ref = DocumentRef::try_from(&last).unwrap();

                provider.add_document(Some(last_doc_ref), &last).unwrap();

                doc = Some(last);
            }

            provider.add_document(Some(first_doc_ref), &first).unwrap();

            (provider, doc.unwrap())
        } => false;
        "chaining to the document already chained to by another document"
    )]
    #[test_case(
        {
            let doc_type = UuidV4::new();
            let doc_id = UuidV7::new();

            let mut provider = TestCatalystProvider::default();

            let first_doc_ver = UuidV7::new();
            let first = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(first_doc_ver))
                .build();
            let first_doc_ref = DocumentRef::try_from(&first).unwrap();
            
            let last_doc_ver = helper::get_now_plus_uuidv7(60);
            let last = Builder::new()
                .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
                .with_metadata_field(SupportedField::Id(doc_id))
                .with_metadata_field(SupportedField::Ver(last_doc_ver))
                .with_metadata_field(SupportedField::Chain(
                    // -2
                    Chain::new(-2, Some(first_doc_ref.clone()))
                ))
                .build();
            let last_doc_ref = DocumentRef::try_from(&last).unwrap();

            provider.add_document(Some(first_doc_ref), &first).unwrap();
            provider.add_document(Some(last_doc_ref), &last).unwrap();

            (provider, last)
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
