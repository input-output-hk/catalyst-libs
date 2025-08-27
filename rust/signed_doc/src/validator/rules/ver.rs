//! Validator for Signed Document Version

use crate::CatalystSignedDocument;

/// Signed Document `ver` field validation rule
pub(crate) struct VerRule;

impl VerRule {
    /// Validates document `ver` field on the timestamps:
    /// 1. document `ver` cannot be smaller than document `id` field
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        let Ok(id) = doc.doc_id() else {
            doc.report().missing_field(
                "id",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };
        let Ok(ver) = doc.doc_ver() else {
            doc.report().missing_field(
                "ver",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };

        if ver < id {
            doc.report().invalid_value(
                "ver",
                &ver.to_string(),
                "ver < id",
                &format!("Document Version {ver} cannot be smaller than Document ID {id}"),
            );
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use test_case::test_case;
    use uuid::{Timestamp, Uuid};

    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField, UuidV7};

    #[test_case(
        || {
            let uuid_v7 = UuidV7::new();
            Builder::new()
                .with_metadata_field(SupportedField::Id(uuid_v7))
                .with_metadata_field(SupportedField::Ver(uuid_v7))
                .build()
        }
        => true;
        "`ver` and `id` are equal"
    )]
    #[test_case(
        || {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .build()
        }
        => true;
        "`ver` greater than `id` are equal"
    )]
    #[test_case(
        || {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let ver = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .build()
        }
        => false;
        "`ver` less than `id` are equal"
    )]
    #[test_case(
        || {
            Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .build()
        }
        => false;
        "missing `ver` field"
    )]
    #[test_case(
        || {
            Builder::new()
                .with_metadata_field(SupportedField::Ver(UuidV7::new()))
                .build()
        }
        => false;
        "missing `id` field"
    )]
    #[tokio::test]
    async fn ver_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
        let doc = doc_gen();

        VerRule.check(&doc).await.unwrap()
    }
}
