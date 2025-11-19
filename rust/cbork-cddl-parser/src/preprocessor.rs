//! A CDDL AST preprocessor.
//!
//! - Validates the root rule of the AST to be a `cddl` rule.
//! - Filters out all rules that are not `expr` rules.
//! - (TODO) Resolve #include and #import directives, by just adding the imported rules
//!   into the final expression list

use anyhow::{anyhow, ensure};
use pest::{RuleType, iterators::Pair};

use crate::parser::{Ast, cddl, rfc_8610, rfc_9165};

/// Processes the AST.
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<Ast> {
    match ast {
        Ast::Rfc8610(ast) => {
            process_root_and_filter(ast, rfc_8610::Rule::cddl, rfc_8610::Rule::expr)
                .map(Ast::Rfc8610)
        },
        Ast::Rfc9165(ast) => {
            process_root_and_filter(ast, rfc_9165::Rule::cddl, rfc_9165::Rule::expr)
                .map(Ast::Rfc9165)
        },
        Ast::Cddl(ast) => {
            process_root_and_filter(ast, cddl::Rule::cddl, cddl::Rule::expr).map(Ast::Cddl)
        },
    }
}

/// Process the root rule of the AST and filter out all non `expected_rule` rules.
fn process_root_and_filter<R: RuleType>(
    ast: Vec<Pair<'_, R>>,
    root_rule: R,
    expected_rule: R,
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
