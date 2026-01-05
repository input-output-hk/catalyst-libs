//! Tests for 'Contest Ballot' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_ballot>

use catalyst_signed_doc::{
    CatalystSignedDocument, doc_types,
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc, contest_ballot_doc,
        contest_parameters_doc, contest_parameters_form_template_doc, proposal_doc,
        proposal_form_template_doc,
    },
    validator::Validator,
};
use test_case::test_case;

use crate::{ContestBallotRule, contest_ballot::ballot::ContestBallot};

#[test_case(
    |provider| {
        let brand = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let parameters = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = proposal_form_template_doc(&parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let proposal = proposal_doc(&template, &parameters, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_ballot_doc(&proposal, &parameters, provider)
    }
    => true
    ;
    "valid document"
)]
fn contest_ballot(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider).unwrap();

    let mut validator = Validator::new();
    validator.extend_rules_per_document(doc_types::CONTEST_BALLOT.clone(), ContestBallotRule {});

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
