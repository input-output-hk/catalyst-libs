//! Identifier Tests
// cspell: words aname groupsocket typesocket groupsocket

mod common;
use common::{identifiers::*, Rule};

/// Check if the name components pass properly.
#[test]
fn check_name_start_characters() {
    let passes = ('\u{0}'..='\u{ff}')
        .filter(|x| x.is_ascii_alphabetic() || matches!(x, '@' | '_' | '$'))
        .map(String::from)
        .collect::<Vec<_>>();
    let fails = ('\u{0}'..='\u{ff}')
        .filter(|x| !x.is_ascii_alphabetic() && !matches!(x, '@' | '_' | '$'))
        .map(String::from)
        .collect::<Vec<_>>();

    common::check_tests_rule(Rule::NAME_START, &passes, &fails);
}

/// Check if the name components pass properly.
#[test]
fn check_name_end_characters() {
    let passes = ('\u{0}'..='\u{ff}')
        .filter(|x| x.is_ascii_alphabetic() || x.is_ascii_digit() || matches!(x, '@' | '_' | '$'))
        .map(String::from)
        .collect::<Vec<_>>();
    let fails = ('\u{0}'..='\u{ff}')
        .filter(|x| {
            !x.is_ascii_alphabetic() && !x.is_ascii_digit() && !matches!(x, '@' | '_' | '$')
        })
        .map(String::from)
        .collect::<Vec<_>>();

    common::check_tests_rule(Rule::NAME_END, &passes, &fails);
}

/// Test if the `id` rule passes properly.
#[test]
fn check_id() {
    common::check_tests_rule(Rule::id_TEST, ID_PASSES, ID_FAILS);
}
