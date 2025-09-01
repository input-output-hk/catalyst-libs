//! Validator for Signed Document ID

use std::time::{Duration, SystemTime};

use anyhow::Context;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Signed Document `id` field validation rule
#[derive(Debug)]
pub(crate) struct IdRule;

impl IdRule {
    /// Validates document `id` field on the timestamps:
    /// 1. If `provider.future_threshold()` not `None`, document `id` cannot be too far in
    ///    the future (`future_threshold` arg) from `SystemTime::now()` based on the
    ///    provide threshold
    /// 2. If `provider.future_threshold()` not `None`, document `id` cannot be too far
    ///    behind (`past_threshold` arg) from `SystemTime::now()` based on the provide
    ///    threshold
    #[allow(clippy::unused_async)]
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let Ok(id) = doc.doc_id() else {
            doc.report().missing_field(
                "id",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };

        let mut is_valid = true;

        let (id_time_secs, id_time_nanos) = id
            .uuid()
            .get_timestamp()
            .ok_or(anyhow::anyhow!("Document `id` field must be a UUIDv7"))?
            .to_unix();

        let Some(id_time) =
            SystemTime::UNIX_EPOCH.checked_add(Duration::new(id_time_secs, id_time_nanos))
        else {
            doc.report().invalid_value(
                    "id",
                    &id.to_string(),
                    "Must a valid duration since `UNIX_EPOCH`",
                    "Cannot instantiate a valid `SystemTime` value from the provided `id` field timestamp.",
                );
            return Ok(false);
        };

        let now = SystemTime::now();

        if let Ok(id_age) = id_time.duration_since(now) {
            // `now` is earlier than `id_time`
            if let Some(future_threshold) = provider.future_threshold() {
                if id_age > future_threshold {
                    doc.report().invalid_value(
                        "id",
                        &id.to_string(),
                        "id < now + future_threshold",
                        &format!("Document Version timestamp {id} cannot be too far in future (threshold: {future_threshold:?}) from now: {now:?}"),
                    );
                    is_valid = false;
                }
            }
        } else {
            // `id_time` is earlier than `now`
            let id_age = now
                .duration_since(id_time)
                .context("BUG! `id_time` must be earlier than `now` at this place")?;

            if let Some(past_threshold) = provider.past_threshold() {
                if id_age > past_threshold {
                    doc.report().invalid_value(
                        "id",
                        &id.to_string(),
                        "id > now - past_threshold",
                        &format!("Document Version timestamp {id} cannot be too far behind (threshold: {past_threshold:?}) from now: {now:?}",),
                    );
                    is_valid = false;
                }
            }
        }

        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
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
    async fn id_test(
        doc_gen: impl FnOnce(&TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let provider = TestCatalystProvider::default();
        let doc = doc_gen(&provider);

        IdRule.check(&doc, &provider).await.unwrap()
    }
}
