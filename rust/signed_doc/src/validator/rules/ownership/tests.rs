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
    validator::rules::utils::create_dummy_key_pair, ContentType,
};

const NO_AUTHOR: usize = 0;
const ONE_AUTHOR: usize = 1;

const NO_COLLABORATORS: usize = 0;
const THREE_COLLABORATORS: usize = 3;

#[derive(Clone)]
struct CatalystAuthorId {
    sk: ed25519_dalek::SigningKey,
    kid: CatalystId,
}

type CatDoc = CatalystSignedDocument;

type DocId = UuidV7;

type Authors = Vec<CatalystAuthorId>;

type Collaborators = Vec<CatalystAuthorId>;

impl CatalystAuthorId {
    fn new() -> Self {
        let (sk, _, kid) = create_dummy_key_pair(RoleId::Role0);
        Self { sk, kid }
    }
}

fn doc_builder(
    doc_id: UuidV7,
    doc_ver: UuidV7,
    authors: Authors,
    collaborators: Collaborators,
) -> (UuidV7, Authors, CatalystSignedDocument) {
    let mut doc_builder = Builder::new()
        .with_metadata_field(SupportedField::Id(doc_id))
        .with_metadata_field(SupportedField::Ver(doc_ver))
        .with_metadata_field(SupportedField::Type(UuidV4::new().into()))
        .with_metadata_field(SupportedField::ContentType(ContentType::Json));

    if !collaborators.is_empty() {
        let collaborators = collaborators
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
    collaborators: Collaborators,
) -> CatalystSignedDocument {
    let (_, _, new_doc) = doc_builder(doc_id, UuidV7::new(), authors, collaborators);
    new_doc
}

fn gen_original_doc_and_provider(
    num_authors: usize,
    num_collaborators: usize,
) -> (CatDoc, DocId, Authors, Collaborators) {
    let authors = gen_authors(num_authors);
    let collaborators = gen_authors(num_collaborators);
    let doc_id = UuidV7::new();
    let doc_ver_1 = UuidV7::new();
    let (_, _, doc_1) = doc_builder(doc_id, doc_ver_1, authors.clone(), collaborators.clone());
    (doc_1, doc_id, authors, collaborators)
}

#[test_case(
    |_provider| {
        let (doc_1, _, _, _) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABORATORS);
        doc_1
    } => true ;
    "First Version Catalyst Signed Document has only one author"
)]
#[test_case(
    |provider| {
        let (doc_1, doc_id, authors, _) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABORATORS);
        provider.add_document(None, &doc_1).unwrap();
        gen_next_ver_doc(doc_id, authors, Vec::new())
    } => true ;
    "Latest Version Catalyst Signed Document has the same author as the first version"
)]
#[test_case(
    |_provider| {
        let (doc_1, _doc_id, _authors, _) = gen_original_doc_and_provider(NO_AUTHOR,NO_COLLABORATORS);
        doc_1
    } => false ;
    "First Version Unsigned Catalyst Document fails"
)]
#[test_case(
    |provider| {
        let (doc_1, doc_id, _authors, _) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABORATORS);
        provider.add_document(None, &doc_1).unwrap();
        let other_author = gen_authors(ONE_AUTHOR);
        gen_next_ver_doc(doc_id, other_author, Vec::new())
    } => false ;
    "Latest Catalyst Signed Document has a different author from the first version"
)]
#[tokio::test]
async fn simple_author_rule_test(
    test_case_fn: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let rule = DocumentOwnershipRule {
        allow_collaborators: false,
    };

    let mut provider = TestCatalystProvider::default();
    let doc = test_case_fn(&mut provider);

    rule.check(&doc, &provider).await.unwrap()
}

#[test_case(
    |_provider| {
        let (doc_1, _, _, _) = gen_original_doc_and_provider(ONE_AUTHOR,NO_COLLABORATORS);
        doc_1
    } => true ;
    "First Version Catalyst Signed Document has the only one author"
)]
#[test_case(
    |provider| {
        let (doc_1, doc_id, mut authors, collaborators) = gen_original_doc_and_provider(ONE_AUTHOR,THREE_COLLABORATORS);
        provider.add_document(None, &doc_1).unwrap();
        authors.extend_from_slice(&collaborators);
        gen_next_ver_doc(doc_id, authors, Vec::new())
    } => true ;
    "Latest Version Catalyst Signed Document signed by first author and all collaborators"
)]
#[allow(clippy::indexing_slicing)]
#[test_case(
    |provider| {
        let (doc_1, doc_id, _, collaborators) = gen_original_doc_and_provider(ONE_AUTHOR,THREE_COLLABORATORS);
        provider.add_document(None, &doc_1).unwrap();

        let random_collaborator = collaborators[thread_rng().gen_range(0..THREE_COLLABORATORS)].clone();
        gen_next_ver_doc(doc_id, vec![random_collaborator], Vec::new())
    } => true ;
    "Latest Version Catalyst Signed Document signed by collaborator"
)]
#[test_case(
    |_provider| {
        let (doc_1, _doc_id, _authors, _) = gen_original_doc_and_provider(NO_AUTHOR,NO_COLLABORATORS);
        doc_1
    } => false ;
    "First Version Unsigned Catalyst Document fails"
)]
#[test_case(
    |provider| {
        let (doc_1, doc_id, _authors, collaborators) = gen_original_doc_and_provider(ONE_AUTHOR,THREE_COLLABORATORS);
        provider.add_document(None, &doc_1).unwrap();
        let other_authors = gen_authors(ONE_AUTHOR);
        gen_next_ver_doc(doc_id, other_authors, collaborators)
    } => false ;
    "Latest Catalyst Signed Document signed by unexpected author"
)]
#[tokio::test]
async fn author_with_collaborators_rule_test(
    test_case_fn: impl FnOnce(&mut TestCatalystProvider) -> CatalystSignedDocument
) -> bool {
    let rule = DocumentOwnershipRule {
        allow_collaborators: true,
    };

    let mut provider = TestCatalystProvider::default();
    let doc = test_case_fn(&mut provider);

    rule.check(&doc, &provider).await.unwrap()
}
