//! A parser for CDDL using the [pest](https://github.com/pest-parser/pest).
//! Utilized for parsing in accordance with RFC-8610, RFC-9165.

use pest::{iterators::Pairs, Parser};

use crate::Extension;

/// RFC-8610 parser.
#[allow(missing_docs)]
pub(crate) mod rfc_8610 {
    /// A Pest parser for RFC-8610.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    pub(crate) struct Parser;
}

/// RFC-9165 parser.
#[allow(missing_docs)]
pub(crate) mod rfc_9165 {
    /// A Pest parser for RFC-9165.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    #[grammar = "grammar/rfc_9165.pest"]
    pub(crate) struct Parser;
}

/// Full CDDL syntax parser.
#[allow(missing_docs)]
pub(crate) mod cddl {
    /// A Pest parser for a full CDDL syntax.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    #[grammar = "grammar/rfc_9165.pest"]
    #[grammar = "grammar/cddl_modules.pest"]
    pub(crate) struct Parser;
}

/// Full CDDL syntax test parser.
/// Parser with DEBUG rules. These rules are only used in tests.
#[allow(missing_docs)]
pub mod cddl_test {
    pub use pest::Parser;
    /// A Pest test parser for a full CDDL syntax.
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/rfc_8610.pest"]
    #[grammar = "grammar/rfc_9165.pest"]
    #[grammar = "grammar/cddl_modules.pest"]
    #[grammar = "grammar/cddl_test.pest"] // Ideally this would only be used in tests.
    pub struct CDDLTestParser;
}

/// CDDL Standard Postlude - read from an external file
const POSTLUDE: &str = include_str!("grammar/postlude.cddl");

/// Abstract Syntax Tree (AST) representing parsed CDDL syntax.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Ast<'a> {
    /// Represents the AST for RFC-8610 CDDL rules.
    Rfc8610(Pairs<'a, rfc_8610::Rule>),
    /// Represents the AST for RFC-9165 CDDL rules.
    Rfc9165(Pairs<'a, rfc_9165::Rule>),
    /// Represents the AST for CDDL Modules rules.
    Cddl(Pairs<'a, cddl::Rule>),
}

/// Parses and checks semantically a CDDL input string.
///
/// # Arguments
///
/// * `input` - A string containing the CDDL input to be parsed.
///
/// # Returns
///
/// Returns `Ok(())` if parsing is successful, otherwise returns an `Err` containing
/// a boxed `CDDLError` indicating the parsing error.
///
/// # Errors
///
/// This function may return an error in the following cases:
///
/// - If there is an issue with parsing the CDDL input.
pub(crate) fn parse_cddl<'a>(
    input: &'a mut String, extension: &Extension,
) -> anyhow::Result<Ast<'a>> {
    input.push_str("\n\n");
    input.push_str(POSTLUDE);

    let ast = match extension {
        Extension::RFC8610 => {
            rfc_8610::Parser::parse(rfc_8610::Rule::cddl, input).map(Ast::Rfc8610)?
        },
        Extension::RFC9165 => {
            rfc_9165::Parser::parse(rfc_9165::Rule::cddl, input).map(Ast::Rfc9165)?
        },
        Extension::CDDL => cddl::Parser::parse(cddl::Rule::cddl, input).map(Ast::Cddl)?,
    };
    Ok(ast)
}
