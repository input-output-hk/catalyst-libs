//! Rules Tests
// cspell: words GENERICARG bigfloat ASSIGNG GROUPNAME tstr genericarg GENERICARG
// cspell: words assigng assignt ASSIGNT GENERICPARM genericparm

mod common;
use common::{Rule, rules::*, type_declarations::*};

/// Test if the `genericarg` rule passes properly.
/// This uses a special rule in the Grammar to test `genericarg` exhaustively.
#[test]
fn check_genericarg() {
    common::check_tests_rule(Rule::genericarg_TEST, GENERICARG_PASSES, GENERICARG_FAILS);
}

/// Test if the `genericparm` rule passes properly.
/// This uses a special rule in the Grammar to test `genericparm` exhaustively.
#[test]
fn check_genericparm() {
    common::check_tests_rule(
        Rule::genericparm_TEST,
        GENERICPARM_PASSES,
        GENERICPARM_FAILS,
    );
}

/// Test if the `assigng` rule passes properly.
/// This uses a special rule in the Grammar to test `assigng` exhaustively.
#[test]
fn check_assigng() {
    common::check_tests_rule(Rule::assigng_TEST, ASSIGNG_PASSES, ASSIGNG_FAILS);
}

/// Test if the `assignt` rule passes properly.
/// This uses a special rule in the Grammar to test `assignt` exhaustively.
#[test]
fn check_assignt() {
    common::check_tests_rule(Rule::assignt_TEST, ASSIGNT_PASSES, ASSIGNT_FAILS);
}

/// Test if the `typename` rule passes properly.
/// This uses a special rule in the Grammar to test `typename` exhaustively.
#[test]
fn check_typename() {
    common::check_tests_rule(Rule::typename_TEST, TYPENAME_PASSES, TYPENAME_FAILS);
}

/// Test if the `groupname` rule passes properly.
/// This uses a special rule in the Grammar to test `groupname` exhaustively.
#[test]
fn check_groupname() {
    common::check_tests_rule(Rule::groupname_TEST, GROUPNAME_PASSES, GROUPNAME_FAILS);
}

/// Test if the `rule` rule passes properly for type variant.
#[test]
fn check_rule_type_composition() {
    let typename_iter = [TYPENAME_PASSES, TYPENAME_FAILS]
        .into_iter()
        .flatten()
        .enumerate();

    let assign_iter = ASSIGNT_PASSES.iter();
    let type_iter = [
        TYPE_PASSES,
        TYPE1_PASSES,
        TYPE2_PASSES,
        TYPE_FAILS,
        TYPE1_FAILS,
        TYPE2_FAILS,
    ]
    .into_iter()
    .flatten()
    .enumerate();

    let rules_iter = typename_iter.zip(assign_iter).zip(type_iter).map(
        |(((i, typename), assign), (k, r#type))| {
            let is_passes = i < TYPENAME_PASSES.len()
                && k < TYPE_PASSES.len() + TYPE1_PASSES.len() + TYPE2_PASSES.len();
            let input = [typename.to_owned(), assign.to_owned(), r#type.to_owned()].join(" ");
            (input, is_passes)
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

    common::check_tests_rule(Rule::expr_TEST, &passes, &fails);
}

/// Test if the `rule` rule passes properly for group variant.
#[test]
fn check_rule_group() {
    common::check_tests_rule(Rule::expr_TEST, RULE_GROUP_PASSES, RULE_GROUP_FAILS);
}
