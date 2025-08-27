//! Validator for Signed Document Version

use crate::{
    providers::CatalystSignedDocumentProvider, CatalystSignedDocument, DocLocator, DocumentRef,
};

/// Signed Document `ver` field validation rule
pub(crate) struct VerRule;

impl VerRule {
    /// Validates document `ver` field on the timestamps:
    /// 1. document `ver` cannot be smaller than document `id` field
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
        let Ok(ver) = doc.doc_ver() else {
            doc.report().missing_field(
                "ver",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };

        let mut is_valid = true;

        if ver < id {
            doc.report().invalid_value(
                "ver",
                &ver.to_string(),
                "ver < id",
                &format!("Document Version {ver} cannot be smaller than Document ID {id}"),
            );
            is_valid = false;
        }

        if ver != id {
            let first_submited_doc = DocumentRef::new(id, id, DocLocator::default());
            if provider.try_get_doc(&first_submited_doc).await?.is_none() {
                doc.report().functional_validation(
                    &format!("`ver` and `id` are not equal, ver: {ver}, id: {id}. Document with `id` and `ver` being equal MUST exist"),
                    "Cannot get a first version document from the provider, document for which `id` and `ver` are equal.",
                );
                is_valid = false;
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
        builder::tests::Builder, metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider, UuidV7,
    };

    #[test_case(
        |_| {
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
        |provider| {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let first_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(id))
                .build();
            provider.add_document(None, &first_doc).unwrap();

            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
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
        |provider| {
            let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let first_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(id))
                .build();
            provider.add_document(None, &first_doc).unwrap();

            let ver = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
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
        |_| {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .build()
        }
        => false;
        "missing first version document"
    )]
    #[test_case(
        |_| {
            Builder::new()
                .with_metadata_field(SupportedField::Id(UuidV7::new()))
                .build()
        }
        => false;
        "missing `ver` field"
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
    async fn ver_test(
        doc_gen: impl FnOnce(&mut TestCatalystSignedDocumentProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystSignedDocumentProvider::default();
        let doc = doc_gen(&mut provider);

        VerRule.check(&doc, &provider).await.unwrap()
    }
}
