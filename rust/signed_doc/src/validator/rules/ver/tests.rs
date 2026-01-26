use chrono::Utc;
use test_case::test_case;
use uuid::{Timestamp, Uuid};

use super::*;
use crate::{
    builder::tests::Builder,
    metadata::SupportedField,
    providers::{TimeThresholdProvider, tests::TestCatalystProvider},
    uuid::{UuidV4, UuidV7},
};

#[test_case(
    |_| {
        let uuid_v7 = UuidV7::new();
        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(uuid_v7))
            .with_metadata_field(SupportedField::Ver(uuid_v7))
            .build()
    }
    => true;
    "`ver` and `id` are equal"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let now = Utc::now().timestamp();

        let id = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now - 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        let first_doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build();
        provider.add_document(&first_doc).unwrap();

        let ver = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now + 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .build()
    }
    => true;
    "`ver` greater than `id`"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let now = Utc::now().timestamp();
        let past_threshold_secs = i64::try_from(provider.past_threshold().unwrap().as_secs()).unwrap_or(0);

        let too_far_in_past = Uuid::new_v7(Timestamp::from_unix_time(
            u64::try_from(now - past_threshold_secs - 1).unwrap_or(0),
            0,
            0,
            0,
        ))
        .try_into()
        .unwrap();

        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(too_far_in_past))
            .with_metadata_field(SupportedField::Ver(too_far_in_past))
            .build()
    }
    => false;
    "`ver` too far in past"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let now = Utc::now().timestamp();
        let future_threshold_secs = i64::try_from(provider.future_threshold().unwrap().as_secs()).unwrap_or(0);

        let too_far_in_future = Uuid::new_v7(Timestamp::from_unix_time(
            u64::try_from(now + future_threshold_secs + 1).unwrap_or(0),
            0,
            0,
            0,
        ))
        .try_into()
        .unwrap();

        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(too_far_in_future))
            .with_metadata_field(SupportedField::Ver(too_far_in_future))
            .build()
    }
    => false;
    "`ver` too far in future"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let now = Utc::now().timestamp();

        let id = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now + 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        let first_doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build();
        provider.add_document(&first_doc).unwrap();

        let ver = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now - 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .build()
    }
    => false;
    "`ver` less than `id`"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let now = Utc::now().timestamp();

        let id = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now + 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(id))
            .build();
        provider.add_document(&doc).unwrap();

        let ver_1 = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now + 3).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver_1))
            .build();
        provider.add_document(&doc).unwrap();

        let ver_2 = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now + 2).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        Builder::with_required_fields()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver_2))
            .build()
    }
    => false;
    "`ver` less than `ver` field for the latest known document"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |_| {
        let now = Utc::now().timestamp();

        let id = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now - 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();
        let ver = Uuid::new_v7(Timestamp::from_unix_time(u64::try_from(now + 1).unwrap_or(0), 0, 0, 0))
            .try_into()
            .unwrap();

        Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build()
    }
    => false;
    "missing first version document"
)]
#[test_case(
    |_| {
        Builder::new()
            .with_metadata_field(SupportedField::Id(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build()
    }
    => false;
    "missing `ver` field"
)]
#[test_case(
    |_| {
        Builder::new()
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .build()
    }
    => false;
    "missing `id` field"
)]
fn ver_test(doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider);

    VerRule::check_inner(&doc, &provider).unwrap();
    println!("{:?}", doc.report());
    !doc.report().is_problematic()
}
