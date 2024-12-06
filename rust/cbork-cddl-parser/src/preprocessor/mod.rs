//! A CDDL AST preprocessor.
//!
//! - Validates the root rule of the AST to be a `cddl` rule.
//! - Filters out all rules that are not `expr` rules.
//! - (TODO) Resolve #include and #import directives, by just adding the imported rules
//!   into the final expression list

mod cddl_rule;

use anyhow::{anyhow, ensure};
use cddl_rule::CddlRule;

use crate::parser::{cddl, rfc_8610, rfc_9165, Ast};

/// Processes the AST.
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<()> {
    match ast {
        Ast::Rfc8610(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_impl(&validated_and_filtered_ast)?;
        },
        Ast::Rfc9165(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_impl(&validated_and_filtered_ast)?;
        },
        Ast::Cddl(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_impl(&validated_and_filtered_ast)?;
        },
    }

    Ok(())
}

/// Validate the root rule of the AST and filter out all non `expected_rule` rules.
fn validate_root_and_filter(ast: Vec<impl CddlRule>) -> anyhow::Result<Vec<impl CddlRule>> {
    let mut ast_iter = ast.into_iter();
    let ast_root = ast_iter.next().ok_or(anyhow!("Empty AST."))?;
    ensure!(
        ast_root.is_cddl() && ast_iter.next().is_none(),
        "AST must have only one root rule, which must be a `cddl` rule."
    );
    Ok(ast_root.inner().filter(CddlRule::is_expr).collect())
}

/// Something
#[allow(clippy::unnecessary_wraps)]
fn process_impl(ast: &Vec<impl CddlRule>) -> anyhow::Result<()> {
    for _expr in ast {}
    Ok(())
}
