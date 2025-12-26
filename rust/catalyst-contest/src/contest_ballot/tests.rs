//! Tests for 'Contest Ballot' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot>

use catalyst_signed_doc::{
    CatalystSignedDocument, providers::tests::TestCatalystProvider,
    tests_utils::contest_ballot_doc, validator::Validator,
};
use test_case::test_case;

use crate::{ContestBallotRule, contest_ballot::ballot::ContestBallot};

#[test_case(
    |provider| {
        let proposal = todo!();

        let contest_parameters = todo!();

        contest_ballot_doc(&proposal, &contest_parameters, provider)
    }
    => true
    ;
    "valid document"
)]
// TODO: FIXME: More cases!
fn contest_ballot(
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
    let contest_ballot = ContestBallot::new(&doc, &provider).unwrap();
    assert_eq!(is_valid, !contest_ballot.report().is_problematic());
    println!("{:?}", contest_ballot.report());

    is_valid
}
