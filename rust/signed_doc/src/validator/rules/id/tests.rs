use std::time::SystemTime;

use test_case::test_case;
use uuid::{Timestamp, Uuid};

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    UuidV7,
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
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let to_far_in_past = Uuid::new_v7(Timestamp::from_unix_time(
                now - provider.past_threshold().unwrap().as_secs() - 1,
                0,
                0,
                0,
            ))
            .try_into()
            .unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Id(to_far_in_past))
            .build()
    }
    => false;
    "`id` to far in past"
)]
#[test_case(
    #[allow(clippy::arithmetic_side_effects)]
    |provider| {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let to_far_in_future = Uuid::new_v7(Timestamp::from_unix_time(
                now + provider.future_threshold().unwrap().as_secs() + 1,
                0,
                0,
                0,
            ))
            .try_into()
            .unwrap();
        Builder::new()
            .with_metadata_field(SupportedField::Id(to_far_in_future))
            .build()
    }
    => false;
    "`id` to far in future"
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
async fn id_test(doc_gen: impl FnOnce(&TestCatalystProvider) -> CatalystSignedDocument) -> bool {
    let provider = TestCatalystProvider::default();
    let doc = doc_gen(&provider);

    IdRule.check(&doc, &provider).await.unwrap()
}
