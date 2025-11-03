//! Validator for Signed Document ID

#[cfg(test)]
mod tests;

use chrono::{Duration, TimeZone, Utc};

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Signed Document `id` field validation rule
#[derive(Debug)]
pub(crate) struct IdRule;

impl IdRule {
    /// Validates document `id` field on the timestamps:
    /// 1. If `provider.future_threshold()` not `None`, document `id` cannot be too far in
    ///    the future (`future_threshold` arg) from `Utc::now()` based on the provided
    ///    threshold
    /// 2. If `provider.past_threshold()` not `None`, document `id` cannot be too far
    ///    behind (`past_threshold` arg) from `Utc::now()` based on the provided threshold
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

        let id_time = Utc
            .timestamp_opt(i64::try_from(id_time_secs).unwrap_or(0), id_time_nanos)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp in document `id` field"))?;

        let now = Utc::now();

        let diff = id_time.signed_duration_since(now);

        if diff.num_nanoseconds().unwrap_or(0) > 0 {
            // id_time is in the future
            if let Some(future_threshold) = provider.future_threshold() {
                let threshold = Duration::from_std(future_threshold)?;
                if diff > threshold {
                    doc.report().invalid_value(
                        "id",
                        &id.to_string(),
                        "id < now + future_threshold",
                        &format!(
                            "Document Version timestamp {id} cannot be too far in future (threshold: {threshold:?}) from now: {now:?}"
                        ),
                    );
                    is_valid = false;
                }
            }
        } else {
            // id_time is in the past
            // make positive duration
            let id_age = diff.abs();

            if let Some(past_threshold) = provider.past_threshold() {
                let threshold = Duration::from_std(past_threshold)?;
                if id_age > threshold {
                    doc.report().invalid_value(
                        "id",
                        &id.to_string(),
                        "id > now - past_threshold",
                        &format!(
                            "Document Version timestamp {id} cannot be too far behind (threshold: {threshold:?}) from now: {now:?}"
                        ),
                    );
                    is_valid = false;
                }
            }
        }

        Ok(is_valid)
    }
}
