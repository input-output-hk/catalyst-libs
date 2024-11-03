//! A parser for CDDL using the [pest](https://github.com/pest-parser/pest).
//! Utilized for parsing in accordance with RFC-8610, RFC-9165.

/// RFC-8610 parser.
pub mod rfc_8610 {
    /// A Pest parser for RFC-8610.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    pub struct Parser;
}

/// RFC-9165 parser.
pub mod rfc_9165 {
    /// A Pest parser for RFC-9165.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    #[grammar = "grammar/rfc_9165.pest"]
    pub struct Parser;
}

/// Full CDDL syntax parser.
pub mod cddl {
    /// A Pest parser for a full CDDL syntax.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    #[grammar = "grammar/rfc_9165.pest"]
    #[grammar = "grammar/cddl_modules.pest"]
    pub struct Parser;
}

/// Full CDDL syntax test parser.
/// Parser with DEBUG rules. These rules are only used in tests.
pub mod cddl_test {
    #[allow(dead_code)]
    /// A Pest test parser for a full CDDL syntax.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    #[grammar = "grammar/rfc_9165.pest"]
    #[grammar = "grammar/cddl_modules.pest"]
    #[grammar = "grammar/cddl_test.pest"] // Ideally this would only be used in tests.
    pub struct CDDLTestParser;
}
