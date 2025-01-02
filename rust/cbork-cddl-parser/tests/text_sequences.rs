mod common;
use common::parser::{check_tests_rule, text_sequences::*, Rule};

#[test]
/// Test if the `S` rule passes properly.
/// This uses a special rule in the Grammar to test whitespace exhaustively.
fn check_s() {
    check_tests_rule(Rule::S_TEST, S_PASSES, S_FAILS);
}

#[test]
/// Test if the `text` rule passes properly.
fn check_text() {
    check_tests_rule(Rule::text_TEST, TEXT_PASSES, TEXT_FAILS);
}
