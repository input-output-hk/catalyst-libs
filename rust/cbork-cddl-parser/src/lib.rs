//! A parser for CDDL, utilized for parsing in accordance with RFC 8610.

mod parser;
mod preprocessor;

/// Represents different grammar extensions for handling CDDL specifications.
pub enum Extension {
    /// RFC8610 ONLY limited grammar.
    RFC8610,
    /// RFC8610 and RFC9165 limited grammar.
    RFC9165,
    /// RFC8610, RFC9165, and CDDL grammar.
    CDDL,
}

/// Verifies semantically a CDDL input string.
///
/// # Errors
///
/// This function may return an error in the following cases:
///
/// - If there is an issue with parsing the CDDL input.
pub fn validate_cddl(input: &mut String, extension: &Extension) -> anyhow::Result<()> {
    let ast = parser::parse_cddl(input, extension)?;
    let _ast = preprocessor::process_ast(ast)?;
    Ok(())
}
