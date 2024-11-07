//! A CDDL AST preprocessor.
//! First processing step, which takes a CDDL `AST` and returning a list of CDDL
//! `Expression`.
//!
//! Preprocessor steps:
//! - Resolve #include and #import directives, by just adding the imported rules into the
//!   final expression list
//! - Resolves all generics by taking the generic arguments and substituting it.

use anyhow::{anyhow, ensure};
use pest::{
    iterators::{Pair, Pairs},
    RuleType,
};

use crate::parser::{cddl, rfc_8610, rfc_9165, Ast};

/// Processes the AST.
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<()> {
    match ast {
        Ast::Rfc8610(ast) => {
            let _exprs = process_root(ast, rfc_8610::Rule::cddl, rfc_8610::Rule::expr)?;
        },
        Ast::Rfc9165(ast) => {
            let _exprs = process_root(ast, rfc_9165::Rule::cddl, rfc_9165::Rule::expr)?;
        },
        Ast::Cddl(ast) => {
            let exprs = process_root(ast, cddl::Rule::cddl, cddl::Rule::expr)?;

            for expr in exprs {
                println!("{:?}", expr.as_rule());
            }
        },
    }
    Ok(())
}

/// Process the root rule of the AST.
/// Returns a vector of expressions of the underlying AST.
fn process_root<R: RuleType>(
    mut ast: Pairs<'_, R>, root_rule: R, expr_rule: R,
) -> anyhow::Result<Vec<Pair<'_, R>>> {
    let ast_root = ast.next().ok_or(anyhow!("Empty AST."))?;
    ensure!(
        ast_root.as_rule() == root_rule && ast.next().is_none(),
        "AST must have only one root rule, which must be a `{root_rule:?}` rule."
    );
    Ok(ast_root
        .into_inner()
        .filter(|pair| pair.as_rule() == expr_rule)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_cddl;

    #[test]
    fn it_works() {
        let mut cddl = include_str!("../../tests/cddl/valid_rfc8610_simple_1.cddl").to_string();

        let ast = parse_cddl(&mut cddl, &crate::Extension::CDDL).unwrap();
        process_ast(ast).unwrap();
    }
}
