//! Test for 'Contest Delegation' document validation part.
//! <https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation>

use catalyst_signed_doc::{
    providers::tests::TestCatalystProvider,
    tests_utils::{
        brand_parameters_doc, brand_parameters_form_template_doc,
        contest_delegation_by_representative_doc, contest_delegation_doc, contest_parameters_doc,
        contest_parameters_form_template_doc, rep_nomination_doc, rep_nomination_form_template_doc,
        rep_profile_doc, rep_profile_form_template_doc,
    },
    validator::Validator,
    *,
};
use test_case::test_case;

use crate::contest_delegation::{ContestDelegation, rule::ContestDelegationRule};

#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let _delegation_by_representative = contest_delegation_by_representative_doc(&rep_nomination, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_delegation_doc(&rep_nomination, &contest, provider)
    }
    => true
    ;
    "valid document"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_delegation_doc(&rep_nomination, &contest, provider)
    }
    => false
    ;
    "missing delegation by representative"
)]
#[test_case(
    |provider| {
        let template = brand_parameters_form_template_doc(provider).inspect(|v| provider.add_document(v).unwrap())?;
        let brand = brand_parameters_doc(&template, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_profile_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_profile = rep_profile_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = contest_parameters_form_template_doc(&brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let contest = contest_parameters_doc(&template, &brand, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let template = rep_nomination_form_template_doc(&contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let rep_nomination = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let _delegation_by_representative = contest_delegation_by_representative_doc(&rep_nomination, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let rep_nomination_latest = rep_nomination_doc(&template, &rep_profile, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        let _delegation_by_representative_2 = contest_delegation_by_representative_doc(&rep_nomination_latest, &contest, provider).inspect(|v| provider.add_document(v).unwrap())?;
        contest_delegation_doc(&rep_nomination, &contest, provider)
    }
    => false
    ;
    "not the latest nomination reference"
)]
#[allow(clippy::unwrap_used)]
fn contest_delegation(
    doc_gen: impl Fn(&mut TestCatalystProvider) -> anyhow::Result<CatalystSignedDocument>
) -> bool {
    let mut provider = TestCatalystProvider::default();
    let doc = doc_gen(&mut provider).unwrap();

    let mut validator = Validator::new();
    validator
        .extend_rules_per_document(doc_types::CONTEST_DELEGATION.clone(), ContestDelegationRule);

    let is_valid = validator.validate(&doc, &provider).unwrap();
    assert_eq!(is_valid, !doc.report().is_problematic());
    println!("{:?}", doc.report());

    // Generate similar `CatalystSignedDocument` instance to have a clean internal problem
    // report
    let doc = doc_gen(&mut provider).unwrap();
    let contest_delegation = ContestDelegation::new(&doc, &provider).unwrap();
    assert_eq!(is_valid, !contest_delegation.report().is_problematic());
    println!("{:?}", contest_delegation.report());

    is_valid
}
