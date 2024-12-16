//! A CDDL AST preprocessor.
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
use cddl_rule::CddlRule;
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

/// Process `expr` rules
fn process_expr_rules(ast: Vec<impl CddlRule>) -> anyhow::Result<ProcessorState> {
    let mut state = ProcessorState::new();
    for expr in ast {
        let mut rules = expr.inner();
        let typename_or_groupname_rule = rules.next().ok_or(anyhow::anyhow!(
            "Invalid `expr` rule, missing `typename` or `groupname` rule"
        ))?;

        if typename_or_groupname_rule.is_typename() {
            let type_name = typename_or_groupname_rule.to_string();
            let cddl_type = process_expr_part_1_rule(rules)?;
            state.insert(type_name, cddl_type);
        }
    }
    Ok(state)
}

/// Process `expr` part 1 rule (which includes `typename` rule)
fn process_expr_part_1_rule(
    mut expr_rules: impl Iterator<Item = impl CddlRule>,
) -> anyhow::Result<CddlType> {
    let rule = expr_rules.next().ok_or(anyhow::anyhow!(
        "Invalid `expr` rule, missing `genericparm` or `assignt` rule"
    ))?;
    if rule.is_genericparm() {
        expr_rules.next().ok_or(anyhow::anyhow!(
            "Invalid `expr` rule, missing `assignt` rule"
        ))?;
    }

    let type_rule = expr_rules
        .next()
        .ok_or(anyhow::anyhow!("Invalid `expr` rule, missing `type` rule"))?;

    process_type_rule(type_rule)
}

/// Process `type` rule
fn process_type_rule(type_rule: impl CddlRule) -> anyhow::Result<CddlType> {
    let rules = type_rule.inner();

    let cddl_types: Vec<_> = rules
        .map(|rule| process_type1_rule(rule))
        .collect::<anyhow::Result<_>>()?;

    if cddl_types.len() > 1 {
        Ok(CddlType::Choice(cddl_types))
    } else {
        cddl_types.into_iter().next().ok_or(anyhow::anyhow!(
            "Invalid `type` rule, missing at least one `type1` rule"
        ))
    }
}

/// Process `type1` rule
fn process_type1_rule(type1_rule: impl CddlRule) -> anyhow::Result<CddlType> {
    let mut rules = type1_rule.inner();

    let type2_rule = rules.next().ok_or(anyhow::anyhow!(
        "Invalid `type1` rule, missing first `type2` rule"
    ))?;
    // TODO: process the rest of the rules

    process_type2_rule(type2_rule)
}

/// Process `type2` rule
fn process_type2_rule(type2_rule: impl CddlRule) -> anyhow::Result<CddlType> {
    let mut rules = type2_rule.inner();

    let rule = rules.next().ok_or(anyhow::anyhow!(
        "Invalid `type2` rule, must have at one inner rule"
    ))?;

    if rule.is_value() {
        // TODO: remove it after processing `value` rule
        Ok(CddlType::Choice(vec![]))
    } else if rule.is_typename() {
        // TODO: need to also process `genericarg` rule
        Ok(CddlType::TypeName(rule.to_string()))
    } else if rule.is_m_type_0() {
        Ok(CddlType::CborType(CborType::MajorType0))
    } else if rule.is_m_type_1() {
        Ok(CddlType::CborType(CborType::MajorType1))
    } else if rule.is_m_type_2() {
        Ok(CddlType::CborType(CborType::MajorType2))
    } else if rule.is_m_type_3() {
        Ok(CddlType::CborType(CborType::MajorType3))
    } else if rule.is_m_type_4() {
        Ok(CddlType::CborType(CborType::MajorType4))
    } else if rule.is_m_type_5() {
        Ok(CddlType::CborType(CborType::MajorType5))
    } else if rule.is_m_type_6() {
        // TODO: remove it after processing `m_type_6` rule
        Ok(CddlType::Choice(vec![]))
    } else if rule.is_m_type_7() {
        // TODO: remove it after processing `m_type_7` rule
        Ok(CddlType::Choice(vec![]))
    } else if rule.is_any() {
        Ok(CddlType::CborType(CborType::Any))
    } else {
        // TODO: after complete parsing all possible variations of `type2` rule need to drop
        // an error instead this
        Ok(CddlType::Choice(vec![]))
    }
}
