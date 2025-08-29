//! Validator for Signed Document Version

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Signed Document `ver` field validation rule
pub(crate) struct VerRule;

impl VerRule {
    /// Validates document `ver` field on the timestamps:
    /// 1. document `ver` cannot be smaller than document `id` field
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
        } else if let Some(last_doc) = provider.try_get_last_doc(id).await? {
            let Ok(last_doc_ver) = last_doc.doc_ver() else {
                doc.report().missing_field(
                    "ver",
                    &format!(
                        "Missing `ver` field in the latest known document, for the the id {id}"
                    ),
                );
                return Ok(false);
            };

            if last_doc_ver >= ver {
                doc.report().functional_validation(
                    &format!("New document ver should be greater that the submitted latest known. New document ver: {ver}, latest known ver: {last_doc_ver}"),
                    &format!("Document's `ver` field should continuously increasing, for the the id {id}"),
                );
                is_valid = false;
            }

            let Ok(last_doc_type) = last_doc.doc_type() else {
                doc.report().missing_field(
                    "type",
                    &format!(
                        "Missing `type` field in the latest known document. Last known document id: {id}, ver: {last_doc_ver}."
                    ),
                );
                return Ok(false);
            };

            let Ok(doc_type) = doc.doc_type() else {
                doc.report().missing_field("type", "Missing `type` field.");
                return Ok(false);
            };

            if last_doc_type != doc_type {
                doc.report().functional_validation(
                    &format!("New document type should be the same that the submitted latest known. New document type: {doc_type}, latest known ver: {last_doc_type}"),
                    &format!("Document's type should be the same for all documents with the same id {id}"),
                );
                is_valid = false;
            }
        } else if ver != id {
            doc.report().functional_validation(
                &format!("`ver` and `id` are not equal, ver: {ver}, id: {id}. Document with `id` and `ver` being equal MUST exist"),
                "Cannot get a first version document from the provider, document for which `id` and `ver` are equal.",
            );
            is_valid = false;
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
        UuidV4, UuidV7,
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
        #[allow(clippy::arithmetic_side_effects)]
        |provider| {
            let doc_type = UuidV4::new();
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
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build();
            provider.add_document(None, &first_doc).unwrap();

            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build()
        }
        => true;
        "`ver` greater than `id`"
    )]
    #[test_case(
        #[allow(clippy::arithmetic_side_effects)]
        |provider| {
            let doc_type = UuidV4::new();
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let first_doc = Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(id))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build();
            provider.add_document(None, &first_doc).unwrap();

            let ver = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build()
        }
        => false;
        "`ver` less than `id`"
    )]
    #[test_case(
        #[allow(clippy::arithmetic_side_effects)]
        |provider| {
            let doc_type = UuidV4::new();
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let id = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(id))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build();
            provider.add_document(None, &doc).unwrap();


            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 3, 0, 0, 0))
                .try_into()
                .unwrap();
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build();
            provider.add_document(None, &doc).unwrap();

            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 2, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build()
        }
        => false;
        "`ver` less than `ver` field for of the latest known document"
    )]
    #[test_case(
        #[allow(clippy::arithmetic_side_effects)]
        |_| {
            let doc_type = UuidV4::new();
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
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build()
        }
        => false;
        "missing first version document"
    )]
    #[test_case(
        #[allow(clippy::arithmetic_side_effects)]
        |provider| {
            let doc_type = UuidV4::new();
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
                .with_metadata_field(SupportedField::Type(doc_type.into()))
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
        => false;
        "missing `type` field"
    )]
    #[test_case(
        #[allow(clippy::arithmetic_side_effects)]
        |provider| {
            let doc_type = UuidV4::new();
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
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .build()
        }
        => false;
        "missing `type` field for the latest known document"
    )]
    #[test_case(
        #[allow(clippy::arithmetic_side_effects)]
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
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .build();
            provider.add_document(None, &first_doc).unwrap();

            let ver = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
                .try_into()
                .unwrap();
            Builder::new()
                .with_metadata_field(SupportedField::Id(id))
                .with_metadata_field(SupportedField::Ver(ver))
                .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
                .build()
        }
        => false;
        "diverge `type` field with the latest known document"
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
        doc_gen: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
    ) -> bool {
        let mut provider = TestCatalystProvider::default();
        let doc = doc_gen(&mut provider);

        VerRule.check(&doc, &provider).await.unwrap()
    }
}
