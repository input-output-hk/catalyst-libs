//! A CDDL AST preprocessor.
//!
//! - Validates the root rule of the AST to be a `cddl` rule.
//! - Filters out all rules that are not `expr` rules.
//! - (TODO) Resolve #include and #import directives, by just adding the imported rules
//!   into the final expression list

use anyhow::{anyhow, ensure};
use pest::{iterators::Pair, RuleType};

use crate::parser::{cddl, rfc_8610, rfc_9165, Ast};

/// Processes the AST.
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<()> {
    match ast {
        Ast::Rfc8610(ast) => {
            let validated_and_filtered_ast =
                validate_root_and_filter(ast, rfc_8610::Rule::cddl, rfc_8610::Rule::expr)?;
            process_impl(validated_and_filtered_ast)?;
        },
        Ast::Rfc9165(ast) => {
            let validated_and_filtered_ast =
                validate_root_and_filter(ast, rfc_9165::Rule::cddl, rfc_9165::Rule::expr)?;
            process_impl(validated_and_filtered_ast)?;
        },
        Ast::Cddl(ast) => {
            let validated_and_filtered_ast =
                validate_root_and_filter(ast, cddl::Rule::cddl, cddl::Rule::expr)?;
            process_impl(validated_and_filtered_ast)?;
        },
    }

    Ok(())
}

/// Validate the root rule of the AST and filter out all non `expected_rule` rules.
fn validate_root_and_filter<R: RuleType>(
    ast: Vec<Pair<'_, R>>, root_rule: R, expected_rule: R,
) -> anyhow::Result<Vec<Pair<'_, R>>> {
    let mut ast_iter = ast.into_iter();
    let ast_root = ast_iter.next().ok_or(anyhow!("Empty AST."))?;
    ensure!(
        ast_root.as_rule() == root_rule && ast_iter.next().is_none(),
        "AST must have only one root rule, which must be a `{root_rule:?}` rule."
    );
    Ok(ast_root
        .into_inner()
        .filter(|pair| pair.as_rule() == expected_rule)
        .collect())
}

/// Something
#[allow(clippy::unnecessary_wraps)]
fn process_impl<R: RuleType>(_ast: Vec<Pair<'_, R>>) -> anyhow::Result<()> {
    Ok(())
}
