//! Process `expr` rule

use super::{
    super::{CddlType, ProcessorState},
    types::process_type_rule,
    CddlRule,
};

/// Process `expr` rules
pub(crate) fn process_expr_rules(ast: Vec<impl CddlRule>) -> anyhow::Result<ProcessorState> {
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
pub(crate) fn process_expr_part_1_rule(
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
