//! A CDDL AST processor.
//!
//! - Validates the root rule of the AST to be a `cddl` rule.
//! - Filters out all rules that are not `expr` rules.
//! - (TODO) Resolve #include and #import directives, by just adding the imported rules
//!   into the final expression list
//! - Adds all stardart prelude cddl types <https://datatracker.ietf.org/doc/html/rfc8610#appendix-D/>
//! - Find all unknown type defintions
//! - Forms a validated state `ProcessorState` of type definitions

#![allow(dead_code)]

mod cddl_rule;
mod cddl_type;

use std::collections::HashMap;

use anyhow::{anyhow, ensure};
use cddl_rule::{expr::process_expr_rules, CddlRule};
use cddl_type::{CborType, CddlType, CddlTypeName};

use crate::parser::{cddl, rfc_8610, rfc_9165, Ast};

/// A final processor state which is generated during this step
type ProcessorState = HashMap<CddlTypeName, CddlType>;

/// Processes the AST.
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<ProcessorState> {
    let state = match ast {
        Ast::Rfc8610(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_expr_rules(validated_and_filtered_ast)?
        },
        Ast::Rfc9165(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_expr_rules(validated_and_filtered_ast)?
        },
        Ast::Cddl(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_expr_rules(validated_and_filtered_ast)?
        },
    };
    Ok(state)
}

/// Validate the root rule to be `cddl` rule and filter out all non `expr`
/// rules.
fn validate_root_and_filter(ast: Vec<impl CddlRule>) -> anyhow::Result<Vec<impl CddlRule>> {
    let mut ast_iter = ast.into_iter();
    let ast_root = ast_iter.next().ok_or(anyhow!("Empty AST."))?;
    ensure!(
        ast_root.is_cddl() && ast_iter.next().is_none(),
        "AST must have only one root rule, which must be a `cddl` rule."
    );
    Ok(ast_root.inner().filter(CddlRule::is_expr).collect())
}
