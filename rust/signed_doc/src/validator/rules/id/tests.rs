use chrono::Utc;
use test_case::test_case;
use uuid::{Timestamp, Uuid};

use super::*;
use crate::{
    builder::tests::Builder,
    metadata::SupportedField,
    providers::{TimeThresholdProvider, tests::TestCatalystProvider},
    uuid::UuidV7,
};

#[test_case(
    |_| {
        let uuid_v7 = UuidV7::new();
        Builder::new()
            .with_metadata_field(SupportedField::Id(uuid_v7))
            .build()
    }
    => true;
    "valid id"
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

        Builder::new()
            .with_metadata_field(SupportedField::Id(too_far_in_past))
            .build()
    }
    => false;
    "`id` too far in past"
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

        Builder::new()
            .with_metadata_field(SupportedField::Id(too_far_in_future))
            .build()
    }
    => false;
    "`id` too far in future"
)]
#[test_case(
    |_| {
        Builder::new()
            .with_metadata_field(SupportedField::Ver(UuidV7::new()))
            .build()
    }
    => false;
    "missing `id` field"
)]
#[tokio::test]
fn id_test(doc_gen: impl FnOnce(&TestCatalystProvider) -> CatalystSignedDocument) -> bool {
    let provider = TestCatalystProvider::default();
    let doc = doc_gen(&provider);

    IdRule::check_inner(&doc, &provider).unwrap()
}
