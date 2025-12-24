//! Tests for 'Contest Ballot' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot>

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::tests::TestCatalystProvider, validator::Validator,
};
use test_case::test_case;

use crate::{ContestBallotRule, contest_ballot::ballot::ContestBallot};

#[test_case(
    |provider| {
        todo!()
    }
    => true
    ;
    "valid document"
)]
fn contest_delegation(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider).unwrap();

    let mut validator = Validator::new();
    validator.extend_rules_per_document(doc_types::CONTEST_BALLOT.clone(), ContestBallotRule);

    let is_valid = validator.validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report
    let doc = doc_gen(&mut provider).unwrap();
    let contest_delegation = ContestBallot::new(&doc, &provider).unwrap();
    assert_eq!(is_valid, !contest_delegation.report().is_problematic());
    println!("{:?}", contest_delegation.report());

    is_valid
}
