// cspell: words hexpair rstuvw abcdefghijklmnopqrstuvwyz rstuvw Xhhb Bhcm

mod common;
use common::parser::{byte_sequences::*, check_tests_rule, Rule};

#[test]
/// Test if the `HEX_PAIR` rule passes properly.
fn check_hexpair() {
    check_tests_rule(Rule::HEX_PAIR, HEXPAIR_PASSES, HEXPAIR_FAILS);
}

#[test]
/// Test if the `URL_BASE64` rule passes properly.
fn check_url_base64() {
    check_tests_rule(Rule::URL_BASE64_TEST, URL_BASE64_PASSES, URL_BASE64_FAILS);
}

#[test]
/// Test if the `bytes` rule passes properly.
fn check_bytes() {
    check_tests_rule(Rule::bytes_TEST, BYTES_PASSES, BYTES_FAILS);
}
