//! Original Author Validation Rule

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

/// Original Author Validation Rule
#[derive(Debug)]
pub(crate) struct OriginalAuthorRule;

impl OriginalAuthorRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let doc_id = doc.doc_id()?;
        let Some(original_doc) = provider.try_get_last_doc(doc_id).await? else {
            return Ok(true);
        };
        let is_valid = original_doc.authors() == doc.authors();
        if !is_valid {
            doc.report().functional_validation(
                &format!("New document authors must match the authors from the previous version for Document ID {doc_id}"),
                &format!(
                    "Document's signatures must be identical to previous version for Document ID {doc_id}"
                ),
            );
        }
        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::{
        catalyst_id::{role_index::RoleId, CatalystId},
        uuid::{UuidV4, UuidV7},
    };
    use ed25519_dalek::ed25519::signature::Signer;
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
        ContentType,
    };

    #[derive(Clone)]
    struct CatalystAuthorId {
        sk: ed25519_dalek::SigningKey,
        kid: CatalystId,
    }

    impl CatalystAuthorId {
        fn new() -> Self {
            let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
            let pk = sk.verifying_key();
            let kid = CatalystId::new("cardano", None, pk).with_role(RoleId::Role0);
            Self { sk, kid }
        }
    }

    fn doc_builder(
        doc_id: UuidV7,
        doc_ver: UuidV7,
        authors: [CatalystAuthorId; 3],
    ) -> (UuidV7, [CatalystAuthorId; 3], CatalystSignedDocument) {
        let mut doc_builder = Builder::new()
            .with_metadata_field(SupportedField::Id(doc_id))
            .with_metadata_field(SupportedField::Ver(doc_ver))
            .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
            .with_metadata_field(SupportedField::ContentType(ContentType::Json))
            .with_content(vec![1, 2, 3]);
        for author in &authors {
            doc_builder = doc_builder
                .add_signature(|m| author.sk.sign(&m).to_vec(), author.kid.clone())
                .unwrap();
        }
        (doc_id, authors, doc_builder.build())
    }

    fn gen_authors() -> [CatalystAuthorId; 3] {
        [
            CatalystAuthorId::new(),
            CatalystAuthorId::new(),
            CatalystAuthorId::new(),
        ]
    }

    fn gen_next_ver_doc(
        doc_id: UuidV7,
        authors: [CatalystAuthorId; 3],
    ) -> CatalystSignedDocument {
        let (_, _, new_doc) = doc_builder(doc_id, UuidV7::new(), authors);
        new_doc
    }

    fn gen_original_doc_and_provider() -> (UuidV7, [CatalystAuthorId; 3], TestCatalystProvider) {
        let authors = gen_authors();
        let doc_id = UuidV7::new();
        let doc_ver_1 = UuidV7::new();
        let (_, _, doc_1) = doc_builder(doc_id, doc_ver_1, authors.clone());
        let mut provider = TestCatalystProvider::default();
        provider.add_document(None, &doc_1).unwrap();
        (doc_id, authors, provider)
    }

    #[test_case(
        || {
            let (doc_id, authors, provider) = gen_original_doc_and_provider();
            let doc_2 = gen_next_ver_doc(doc_id, authors);
            (doc_2, provider)
        } => true ;
        "Catalyst Signed Document has the same authors as the previous version"
    )]
    #[test_case(
        || {
            let (doc_id, _, provider) = gen_original_doc_and_provider();
            let other_authors = gen_authors();
            let doc_2 = gen_next_ver_doc(doc_id, other_authors);
            (doc_2, provider)
        } => false ;
        "Catalyst Signed Document has the different authors from the previous version"
    )]
    #[tokio::test]
    async fn original_author_rule_test(
        doc_gen: impl FnOnce() -> (CatalystSignedDocument, TestCatalystProvider)
    ) -> bool {
        let (doc_v2, provider) = doc_gen();

        OriginalAuthorRule.check(&doc_v2, &provider).await.unwrap()
    }
}
