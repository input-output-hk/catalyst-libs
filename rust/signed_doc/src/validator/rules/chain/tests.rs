use catalyst_signed_doc_spec::{is_required::IsRequired, metadata::chain::Chain as ChainSpec};
use catalyst_types::uuid::{UuidV4, UuidV7};
use test_case::test_case;

use super::*;
use crate::{
    Chain, DocType, builder::tests::Builder, metadata::SupportedField,
    providers::tests::TestCatalystProvider,
};

mod helper {
    use catalyst_types::uuid::UuidV7;
    use chrono::{Duration, Utc};
    use uuid::{Timestamp, Uuid};

    pub(super) fn get_now_plus_uuidv7(secs: i64) -> UuidV7 {
        let future_time = Utc::now()
            .checked_add_signed(Duration::seconds(secs))
            .expect("time overflow in future_time calculation");

        let unix_secs = u64::try_from(future_time.timestamp()).unwrap_or(0);
        let nanos = future_time.timestamp_subsec_nanos();

        let ts = Timestamp::from_unix(uuid::NoContext, unix_secs, nanos);
        let uuid = Uuid::new_v7(ts);

        UuidV7::try_from_uuid(uuid).unwrap()
    }
}

#[test]
fn test_without_chaining_documents() {
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
    assert!(rule.check(&doc, &provider).unwrap());
    let rule = ChainRule::Specified { optional: true };
    assert!(rule.check(&doc, &provider).unwrap());
    let rule = ChainRule::Specified { optional: false };
    assert!(!rule.check(&doc, &provider).unwrap());
}

#[test]
fn chain_rule_collaborators_rule_conflict() {
    let chain = ChainSpec {
        required: IsRequired::Optional,
    };
    let collaborators = Collaborators {
        required: IsRequired::Optional,
    };
    ChainRule::new(&chain, &collaborators).unwrap_err();
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
            .with_metadata_field(SupportedField::Chain(
                Chain::new(0, None)
            ))
            .build();
        let first_doc_ref = first.doc_ref().unwrap();

        let last_doc_ver = helper::get_now_plus_uuidv7(60);
        let last = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(last_doc_ver))
            .with_metadata_field(SupportedField::Chain(
                Chain::new(-1, Some(first_doc_ref.clone()))
            ))
            .build();
        provider.add_document(&first).unwrap();

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
            .with_metadata_field(SupportedField::Chain(
                Chain::new(0, None)
            ))
            .build();
        let first_doc_ref = first.doc_ref().unwrap();

        let intermediate_doc_ver = helper::get_now_plus_uuidv7(60);
        let intermediate = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(intermediate_doc_ver))
            .with_metadata_field(SupportedField::Chain(
                Chain::new(1, Some(first_doc_ref.clone()))
            ))
            .build();
        let intermediate_doc_ref = intermediate.doc_ref().unwrap();

        let last_doc_ver = helper::get_now_plus_uuidv7(120);
        let last = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(last_doc_ver))
            .with_metadata_field(SupportedField::Chain(
                Chain::new(-2, Some(intermediate_doc_ref.clone()))
            ))
            .build();
        provider.add_document(&first).unwrap();
        provider.add_document(&intermediate).unwrap();

        (provider, last)
    } => true;
    "valid intermediate chained documents (0, 1, -2)"
)]

fn test_valid_chained_documents(
    (provider, doc): (TestCatalystProvider, CatalystSignedDocument)
) -> bool {
    let rule = ChainRule::Specified { optional: false };

    rule.check(&doc, &provider).unwrap()
}

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
            .with_metadata_field(SupportedField::Chain(
                Chain::new(0, None)
            ))
            .build();
        let first_doc_ref = first.doc_ref().unwrap();

        let last_doc_ver = helper::get_now_plus_uuidv7(60);
        let last = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
            .with_metadata_field(SupportedField::Id(doc_id_another))
            .with_metadata_field(SupportedField::Ver(last_doc_ver))
            .with_metadata_field(SupportedField::Chain(
                Chain::new(-1, Some(first_doc_ref.clone()))
            ))
            .build();
        provider.add_document(&first).unwrap();

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
            .with_metadata_field(SupportedField::Chain(
                Chain::new(0, None)
            ))
            .build();
        let first_doc_ref = first.doc_ref().unwrap();

        // version not greater than first (using an earlier timestamp)
        let last_doc_ver = helper::get_now_plus_uuidv7(-60);
        let last = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type)))
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(last_doc_ver))
            .with_metadata_field(SupportedField::Chain(
                Chain::new(-1, Some(first_doc_ref.clone()))
            ))
            .build();
        provider.add_document(&first).unwrap();

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
            .with_metadata_field(SupportedField::Chain(
                Chain::new(0, None)
            ))
            .build();
        let first_doc_ref = first.doc_ref().unwrap();

        let last_doc_ver = helper::get_now_plus_uuidv7(60);
        let last = Builder::new()
            .with_metadata_field(SupportedField::Type(DocType::from(doc_type_another)))
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(last_doc_ver))
            .with_metadata_field(SupportedField::Chain(
                Chain::new(-1, Some(first_doc_ref.clone()))
            ))
            .build();
        provider.add_document(&first).unwrap();

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
            .with_metadata_field(SupportedField::Chain(
                Chain::new(0, None)
            ))
            .build();
        let first_doc_ref = first.doc_ref().unwrap();

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
        provider.add_document(&first).unwrap();

        (provider, last)
    } => false;
    "not have its absolute height exactly one more than the height of the document being chained to"
)]

fn test_invalid_chained_documents(
    (provider, doc): (TestCatalystProvider, CatalystSignedDocument)
) -> bool {
    let rule = ChainRule::Specified { optional: false };

    rule.check_inner(&doc, &provider).unwrap()
}
