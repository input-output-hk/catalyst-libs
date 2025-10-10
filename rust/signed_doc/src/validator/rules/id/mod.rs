//! Validator for Signed Document ID

#[cfg(test)]
mod tests;

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
