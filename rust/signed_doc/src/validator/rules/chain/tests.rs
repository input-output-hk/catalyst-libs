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
