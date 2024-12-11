//! A CDDL AST preprocessor.
//!
//! - Validates the root rule of the AST to be a `cddl` rule.
//! - Filters out all rules that are not `expr` rules.
//! - (TODO) Resolve #include and #import directives, by just adding the imported rules
//!   into the final expression list
//! - Adds all stardart prelude cddl types <https://datatracker.ietf.org/doc/html/rfc8610#appendix-D/>
//! - Find all unknown type defintions
//! - Forms a validated list of type definitions ()

#![allow(missing_docs, clippy::missing_docs_in_private_items, dead_code)]

mod cddl_rule;
mod cddl_type;

use std::collections::HashMap;

use anyhow::{anyhow, ensure};
use cddl_rule::CddlRule;
use cddl_type::{CddlType, CddlTypeName};

use crate::parser::{cddl, rfc_8610, rfc_9165, Ast};

/// Processes the AST.
pub(crate) fn process_ast(ast: Ast) -> anyhow::Result<()> {
    match ast {
        Ast::Rfc8610(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_expr_rules(validated_and_filtered_ast)?;
        },
        Ast::Rfc9165(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_expr_rules(validated_and_filtered_ast)?;
        },
        Ast::Cddl(ast) => {
            let validated_and_filtered_ast = validate_root_and_filter(ast)?;
            process_expr_rules(validated_and_filtered_ast)?;
        },
    }

    Ok(())
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

/// Process `expr` rules
fn process_expr_rules(ast: Vec<impl CddlRule>) -> anyhow::Result<HashMap<CddlTypeName, CddlType>> {
    let state = HashMap::new();

    for expr in ast {
        // println!("{}", expr.to_string());
        // println!("{expr:?}");

        let mut rules = expr.inner();
        let typename_or_groupname_rule = rules.next().ok_or(anyhow::anyhow!(
            "Invalid `expr` rule, missing `typename` or `groupname` rule"
        ))?;

        if typename_or_groupname_rule.is_typename() {
            let rule = rules.next().ok_or(anyhow::anyhow!(
                "Invalid `expr` rule, missing `genericparm` or `assignt` rule"
            ))?;
            if rule.is_genericparm() {
                rules.next().ok_or(anyhow::anyhow!(
                    "Invalid `expr` rule, missing `assignt` rule"
                ))?;
            }

            let type_rule = rules
                .next()
                .ok_or(anyhow::anyhow!("Invalid `expr` rule, missing `type` rule"))?;
            process_type_rule(type_rule)?;
        }
    }
    Ok(state)
}

/// Process `type` rule
fn process_type_rule(type_rule: impl CddlRule) -> anyhow::Result<()> {
    let mut rules = type_rule.inner();

    let _type1_rule = rules
        .next()
        .ok_or(anyhow::anyhow!("Invalid `type` rule, missing `type1` rule"))?;

    Ok(())
}
