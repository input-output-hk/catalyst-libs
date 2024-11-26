// cspell: words PCHAR pchar BCHAR bchar SESC sesc SCHAR schar fffd fffe

mod common;
use common::Rule;

/// Test if the `WHITESPACE` rule passes properly.
#[test]
fn check_whitespace() {
    common::check_tests_rule(Rule::WHITESPACE, &[" ", "\t", "\r", "\n", "\r\n"], &["not"]);
}

/// Test if the `PCHAR` rule passes properly.
#[test]
fn check_pchar() {
    let passes = ('\u{0}'..='\u{ff}')
        .filter(|x| x >= &' ' && x != &'\u{7f}')
        .map(String::from)
        .collect::<Vec<_>>();
    let fails = ('\u{0}'..='\u{ff}')
        .filter(|x| x < &' ' || x == &'\u{7f}')
        .map(String::from)
        .collect::<Vec<_>>();
    common::check_tests_rule(Rule::PCHAR, &passes, &fails);
}

/// Test if the `BCHAR` rule passes properly.
#[test]
fn check_bchar() {
    let passes = ('\u{0}'..='\u{ff}')
        .filter(|x| {
            (x >= &' ' && !matches!(x, '\t' | '\'' | '\\' | '\u{7f}')) || matches!(x, '\n' | '\r')
        })
        .map(String::from)
        .collect::<Vec<_>>();

    let fails = ('\u{0}'..='\u{ff}')
        .filter(|x| {
            x < &' ' && !matches!(x, '\n' | '\r') || matches!(x, '\t' | '\'' | '\\' | '\u{7f}')
        })
        .map(String::from)
        .collect::<Vec<_>>();

    common::check_tests_rule(Rule::BCHAR, &passes, &fails);
}

/// Test if the `SESC` rule passes properly.
#[test]
fn check_sesc() {
    let passes = (' '..='\u{ff}')
        .filter(|x| x != &'\u{7f}')
        .map(|x| format!("\\{x}"))
        .collect::<Vec<_>>();
    common::check_tests_rule(Rule::SESC, &passes, &["\u{7f}"]);
}

/// Test if the `ASCII_VISIBLE` rule passes properly.
#[test]
fn check_ascii_visible() {
    let passes = (' '..='~').map(String::from).collect::<Vec<_>>();
    common::check_tests_rule(Rule::ASCII_VISIBLE, &passes, &["\r", "\u{80}"]);
}

/// Test if the `SCHAR_ASCII_VISIBLE` rule passes properly.
#[test]
fn check_schar_ascii_visible() {
    let passes = (' '..='~')
        .filter(|c| c != &'"' && c != &'\\')
        .map(String::from)
        .collect::<Vec<_>>();
    common::check_tests_rule(Rule::SCHAR_ASCII_VISIBLE, &passes, &[
        "\"", "\\", "\r", "\u{80}",
    ]);
}

/// Test if the `BCHAR_ASCII_VISIBLE` rule passes properly.
#[test]
fn check_bchar_ascii_visible() {
    let passes = (' '..='~')
        .filter(|c| c != &'\'' && c != &'\\')
        .map(String::from)
        .collect::<Vec<_>>();
    common::check_tests_rule(Rule::BCHAR_ASCII_VISIBLE, &passes, &[
        "'", "\\", "\r", "\u{80}",
    ]);
}

/// Test if the `UNICODE_CHAR` rule passes properly.
#[test]
fn check_unicode() {
    common::check_tests_rule(
        Rule::UNICODE_CHAR,
        &["\u{80}", "\u{10fffd}", "\u{7ffff}"],
        &["\r", "\u{10fffe}"],
    );
}
