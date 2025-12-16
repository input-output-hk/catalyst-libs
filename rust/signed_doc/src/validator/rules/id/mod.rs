//! Validator for Signed Document ID

#[cfg(test)]
mod tests;

use anyhow::Context;
use chrono::{DateTime, Utc};

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// Signed Document `id` field validation rule
#[derive(Debug)]
pub(crate) struct IdRule;

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for IdRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Self::check_inner(doc, provider)
    }
}

impl IdRule {
    /// Validates document `id` field on the timestamps:
    /// 1. If `provider.future_threshold()` not `None`, document `id` cannot be too far in
    ///    the future (`future_threshold` arg) from `Utc::now()` based on the provided
    ///    threshold
    /// 2. If `provider.past_threshold()` not `None`, document `id` cannot be too far
    ///    behind (`past_threshold` arg) from `Utc::now()` based on the provided threshold
    fn check_inner(
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
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

        let Some(id_time) = i64::try_from(id_time_secs)
            .ok()
            .and_then(|id_time_secs| DateTime::from_timestamp(id_time_secs, id_time_nanos))
        else {
            doc.report().invalid_value(
                    "id",
                    &id.to_string(),
                    "Must a valid UTC date time since `UNIX_EPOCH`",
                    "Cannot instantiate a valid `DateTime<Utc>` value from the provided `id` field timestamp.",
                );
            return Ok(false);
        };

        let now = Utc::now();
        let time_delta = id_time.signed_duration_since(now);

        if let Ok(id_age) = time_delta.to_std() {
            // `now` is earlier than `id_time`
            if let Some(future_threshold) = provider.future_threshold()
                && id_age > future_threshold
            {
                doc.report().invalid_value(
                        "id",
                        &id.to_string(),
                        "id < now + future_threshold",
                        &format!("Document ID timestamp {id} cannot be too far in future (threshold: {future_threshold:?}) from now: {now}"),
                    );
                is_valid = false;
            }
        } else {
            // `id_time` is earlier than `now`
            let id_age = time_delta
                .abs()
                .to_std()
                .context("BUG! `id_time` must be earlier than `now` at this place")?;

            if let Some(past_threshold) = provider.past_threshold()
                && id_age > past_threshold
            {
                doc.report().invalid_value(
                        "id",
                        &id.to_string(),
                        "id > now - past_threshold",
                        &format!("Document ID timestamp {id} cannot be too far behind (threshold: {past_threshold:?}) from now: {now:?}",),
                    );
                is_valid = false;
            }
        }

        Ok(is_valid)
    }
}
