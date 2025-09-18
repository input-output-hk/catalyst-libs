//! Ownership Validation Rule testing

use catalyst_types::{
    catalyst_id::{role_index::RoleId, CatalystId},
    uuid::{UuidV4, UuidV7},
};
use ed25519_dalek::ed25519::signature::Signer;
use rand::{thread_rng, Rng};
use test_case::test_case;

use super::*;
use crate::{
    builder::tests::Builder, metadata::SupportedField, providers::tests::TestCatalystProvider,
    ContentType,
};

const NO_AUTHOR: usize = 0;
const ONE_AUTHOR: usize = 1;

const NO_COLLABS: usize = 0;
const THREE_COLLABS: usize = 3;

#[derive(Clone)]
struct CatalystAuthorId {
    sk: ed25519_dalek::SigningKey,
    kid: CatalystId,
}

type CatDoc = CatalystSignedDocument;

type DocId = UuidV7;

type Authors = Vec<CatalystAuthorId>;

type Collabs = Vec<CatalystAuthorId>;

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
    authors: Authors,
    collabs: Collabs,
) -> (UuidV7, Authors, CatalystSignedDocument) {
    let mut doc_builder = Builder::new()
        .with_metadata_field(SupportedField::Id(doc_id))
        .with_metadata_field(SupportedField::Ver(doc_ver))
        .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
        .with_metadata_field(SupportedField::ContentType(ContentType::Json));

    if !collabs.is_empty() {
        let collaborators = collabs
            .into_iter()
            .map(|c| c.kid)
            .collect::<Vec<CatalystId>>();
        doc_builder =
            doc_builder.with_metadata_field(SupportedField::Collaborators(collaborators.into()));
    }

    for author in &authors {
        doc_builder = doc_builder
            .add_signature(|m| author.sk.sign(&m).to_vec(), author.kid.clone())
            .unwrap();
    }
    (doc_id, authors, doc_builder.build())
}

fn gen_authors(size_of: usize) -> Authors {
    (0..size_of).map(|_| CatalystAuthorId::new()).collect()
}

fn gen_next_ver_doc(
    doc_id: UuidV7,
    authors: Authors,
    collabs: Collabs,
) -> CatalystSignedDocument {
    let (_, _, new_doc) = doc_builder(doc_id, UuidV7::new(), authors, collabs);
    new_doc
}

fn gen_original_doc_and_provider(
    num_authors: usize,
    num_collaborators: usize,
) -> (CatDoc, DocId, Authors, Collabs, TestCatalystProvider) {
    let authors = gen_authors(num_authors);
    let collaborators = gen_authors(num_collaborators);
    let doc_id = UuidV7::new();
    let doc_ver_1 = UuidV7::new();
    let (_, _, doc_1) = doc_builder(doc_id, doc_ver_1, authors.clone(), collaborators.clone());
    let provider = TestCatalystProvider::default();
    (doc_1, doc_id, authors, collaborators, provider)
}

#[test_case(
    || {
        let (doc_1, _, _, _, provider) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABS);
        (doc_1, provider)
    } => true ;
    "First Version Catalyst Signed Document has the only one author"
)]
#[test_case(
    || {
        let (doc_1, doc_id, authors, _, mut provider) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABS);
        provider.add_document(None, &doc_1).unwrap();
        let doc_2 = gen_next_ver_doc(doc_id, authors, Vec::new());
        (doc_2, provider)
    } => true ;
    "Latest Version Catalyst Signed Document has the same author as the first version"
)]
#[test_case(
    || {
        let (doc_1, _doc_id, _authors, _, provider) = gen_original_doc_and_provider(NO_AUTHOR,NO_COLLABS);
        (doc_1, provider)
    } => false ;
    "First Version Unsigned Catalyst Document fails"
)]
#[test_case(
    || {
        let (doc_1, doc_id, _authors, _, mut provider) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABS);
        provider.add_document(None, &doc_1).unwrap();
        let other_author = gen_authors(ONE_AUTHOR);
        let doc_2 = gen_next_ver_doc(doc_id, other_author, Vec::new());
        (doc_2, provider)
    } => false ;
    "Latest Catalyst Signed Document has a different author from the first version"
)]
#[tokio::test]
async fn simple_author_rule_test(
    test_case_fn: impl FnOnce() -> (CatalystSignedDocument, TestCatalystProvider)
) -> bool {
    let rule = DocumentOwnershipRule {
        allow_collaborators: false,
    };

    let (doc, provider) = test_case_fn();

    rule.check(&doc, &provider).await.unwrap()
}

#[test_case(
    || {
        let (doc_1, _, _, _, provider) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABS);
        (doc_1, provider)
    } => true ;
    "First Version Catalyst Signed Document has the only one author"
)]
#[test_case(
    || {
        let (doc_1, doc_id, mut authors, collabs, mut provider) = gen_original_doc_and_provider(ONE_AUTHOR,THREE_COLLABS);
        provider.add_document(None, &doc_1).unwrap();
        authors.extend_from_slice(&collabs);
        let doc_2 = gen_next_ver_doc(doc_id, authors, Vec::new());
        (doc_2, provider)
    } => true ;
    "Latest Version Catalyst Signed Document signed by first author and all collaborators"
)]
#[allow(clippy::indexing_slicing)]
#[test_case(
    || {
        let (doc_1, doc_id, _, collabs, mut provider) = gen_original_doc_and_provider(ONE_AUTHOR,THREE_COLLABS);
        provider.add_document(None, &doc_1).unwrap();

        let random_collaborator = collabs[thread_rng().gen_range(0..THREE_COLLABS)].clone();
        let doc_2 = gen_next_ver_doc(doc_id, vec![random_collaborator], Vec::new());
        (doc_2, provider)
    } => true ;
    "Latest Version Catalyst Signed Document signed by collaborator"
)]
#[test_case(
    || {
        let (doc_1, _doc_id, _authors, _, provider) = gen_original_doc_and_provider(NO_AUTHOR,NO_COLLABS);
        (doc_1, provider)
    } => false ;
    "First Version Unsigned Catalyst Document fails"
)]
#[test_case(
    || {
        let (doc_1, doc_id, _authors, collabs, mut provider) = gen_original_doc_and_provider(ONE_AUTHOR,THREE_COLLABS);
        provider.add_document(None, &doc_1).unwrap();
        let other_authors = gen_authors(ONE_AUTHOR);
        let doc= gen_next_ver_doc(doc_id, other_authors, collabs);
        (doc, provider)
    } => false ;
    "Latest Catalyst Signed Document has the unexpected authors"
)]
#[tokio::test]
async fn author_with_collaborators_rule_test(
    test_case_fn: impl FnOnce() -> (CatalystSignedDocument, TestCatalystProvider)
) -> bool {
    let rule = DocumentOwnershipRule {
        allow_collaborators: true,
    };

    let (doc, provider) = test_case_fn();

    rule.check(&doc, &provider).await.unwrap()
}
