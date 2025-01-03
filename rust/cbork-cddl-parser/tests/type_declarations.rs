//! Type Declaration Tests
// cspell: words CTLOP aname groupsocket typesocket RANGEOP tstr ctlop
// cspell: words rangeop RANGEOP

mod common;
use common::{type_declarations::*, Rule};

/// Test if the `ctlop` rule passes properly.
/// This uses a special rule in the Grammar to test `ctlop` exhaustively.
#[test]
fn check_ctlop() {
    common::check_tests_rule(Rule::ctlop_TEST, CTLOP_PASSES, CTLOP_FAILS);
}

/// Test if the `rangeop` rule passes properly.
/// This uses a special rule in the Grammar to test `rangeop` exhaustively.
#[test]
fn check_rangeop() {
    common::check_tests_rule(Rule::rangeop_TEST, RANGEOP_PASSES, RANGEOP_FAILS);
}

/// Test if the `type2` rule passes properly.
/// This uses a special rule in the Grammar to test `type2` exhaustively.
#[test]
fn check_type2() {
    common::check_tests_rule(Rule::type2_TEST, TYPE2_PASSES, TYPE2_FAILS);
}

/// Test if the `type1` rule passes properly.
/// This uses a special rule in the Grammar to test `type1` exhaustively.
#[test]
fn check_type1() {
    common::check_tests_rule(Rule::type1_TEST, TYPE1_PASSES, TYPE1_FAILS);
}

/// Test if the `type1` rule passes properly based on composition of type2 test cases.
#[test]
fn check_type1_composition() {
    let separator_iter = [CTLOP_PASSES, RANGEOP_PASSES].into_iter().flatten();

    let type_iter = [TYPE2_PASSES, TYPE_FAILS, TYPE1_FAILS, TYPE2_FAILS]
        .into_iter()
        .flatten()
        .enumerate();

    let rules_iter = type_iter.clone().zip(separator_iter).zip(type_iter).map(
        |(((i, type_1), separator), (j, type_2))| {
            let is_passed = i < TYPE2_PASSES.len() && j < TYPE2_PASSES.len();
            let input = [type_1.to_owned(), separator.to_owned(), type_2.to_owned()].join(" ");
            (input, is_passed)
        },
    );

    let passes = rules_iter
        .clone()
        .filter(|(_, is_passes)| *is_passes)
        .map(|(input, _)| input)
        .collect::<Vec<_>>();

    let fails = rules_iter
        .filter(|(_, is_passes)| !*is_passes)
        .map(|(input, _)| input)
        .collect::<Vec<_>>();

    common::check_tests_rule(Rule::type1_TEST, &passes, &fails);
}

/// Test if the `type` rule passes properly.
/// This uses a special rule in the Grammar to test `type` exhaustively.
#[test]
fn check_type() {
    common::check_tests_rule(Rule::type_TEST, TYPE_PASSES, TYPE_FAILS);
}

/// Test if the `type` rule passes properly based on composition of type2 test cases.
#[test]
fn check_type_composition() {
    // type2 composition testing
    let type_iter = [TYPE2_PASSES, TYPE_FAILS, TYPE1_FAILS, TYPE2_FAILS]
        .into_iter()
        .flatten()
        .enumerate();

    let rules_iter = type_iter
        .clone()
        .zip(type_iter)
        .map(|((i, test_i), (j, test_j))| {
            let is_passed = i < TYPE2_PASSES.len() && j < TYPE2_PASSES.len();
            let input = [test_i.to_owned(), "/", test_j.to_owned()].join(" ");
            (input, is_passed)
        });

    let passes = rules_iter
        .clone()
        .filter(|(_, is_passes)| *is_passes)
        .map(|(input, _)| input)
        .collect::<Vec<_>>();

    let fails = rules_iter
        .filter(|(_, is_passes)| !*is_passes)
        .map(|(input, _)| input)
        .collect::<Vec<_>>();

    common::check_tests_rule(Rule::type_TEST, &passes, &fails);
}
