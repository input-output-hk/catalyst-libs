//! A CDDL AST processor

mod expr;

use anyhow::{anyhow, ensure};
use pest::{iterators::Pairs, RuleType};

use crate::parser::Ast;

/// A helper generic trait for representing a whole CDDL grammar with all extensions.
trait CddlExpr: RuleType {
    /// `cddl` rule
    const CDDL: Self;
    /// `rule` rule
    const RULE: Self;
}

/// Processes the AST.
#[allow(dead_code)]
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<()> {
    match ast {
        Ast::Rfc8610(pairs) => process_ast_impl(pairs),
        Ast::Rfc9165(pairs) => process_ast_impl(pairs),
        Ast::Cddl(pairs) => process_ast_impl(pairs),
    }
}

/// Process AST implementation
fn process_ast_impl<E: CddlExpr>(mut ast: Pairs<'_, E>) -> anyhow::Result<()> {
    let ast_root = ast.next().ok_or(anyhow!("Empty AST"))?;
    ensure!(
        ast_root.as_rule() == E::CDDL && ast.next().is_none(),
        "AST must have only one root rule, which must be a `cddl` rule."
    );

    let pairs = ast_root.into_inner();

    for pair in pairs {
        if pair.as_rule() == E::RULE {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::parse_cddl, Extension};

    #[test]
    fn test() {
        let mut file = include_str!("../../tests/cddl/valid_rfc8610_simple_1.cddl").to_string();

        let ast = parse_cddl(&mut file, &Extension::CDDL).unwrap();
        process_ast(ast).unwrap();
    }
}
