//! Process `type`, `type1`, `type2` rules

use anyhow::anyhow;

use super::{uint::process_uint_rule, CddlRule};
use crate::preprocessor::{CborType, CddlType};

/// Process `type` rule
pub(crate) fn process_type_rule(type_rule: impl CddlRule) -> anyhow::Result<CddlType> {
    let rules = type_rule.inner();

    let cddl_types: Vec<_> = rules
        .map(|rule| process_type1_rule(rule))
        .collect::<anyhow::Result<_>>()?;

    if cddl_types.len() > 1 {
        Ok(CddlType::Choice(cddl_types))
    } else {
        cddl_types.into_iter().next().ok_or(anyhow!(
            "Invalid `type` rule, missing at least one `type1` rule"
        ))
    }
}

/// Process `type1` rule
pub(crate) fn process_type1_rule(type1_rule: impl CddlRule) -> anyhow::Result<CddlType> {
    let mut rules = type1_rule.inner();

    let type2_rule = rules
        .next()
        .ok_or(anyhow!("Invalid `type1` rule, missing first `type2` rule"))?;
    // TODO: process the rest of the rules

    process_type2_rule(type2_rule)
}

/// Process `type2` rule
#[allow(clippy::similar_names)]
fn process_type2_rule(type2_rule: impl CddlRule) -> anyhow::Result<CddlType> {
    let mut rules = type2_rule.inner();

    let rule = rules
        .next()
        .ok_or(anyhow!("Invalid `type2` rule, must have at one inner rule"))?;

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
        let mut rules = rule.inner();
        let uint_rule = rules
            .next()
            .ok_or(anyhow!("Invalid `m_type_6` rule, must have `uint` rule"))?;

        let tag_number = process_uint_rule(&uint_rule)?;

        let type_rule = rules
            .next()
            .ok_or(anyhow!("Invalid `m_type_6` rule, must have `type` rule"))?;
        let tag_content = process_type_rule(type_rule)?;

        Ok(CddlType::CborType(CborType::MajorType6(
            tag_number,
            tag_content.into(),
        )))
    } else if rule.is_m_type_7() {
        let mut rules = rule.inner();

        let uint_rule = rules
            .next()
            .ok_or(anyhow!("Invalid `m_type_7` rule, must have `uint` rule"))?;
        let val = process_uint_rule(&uint_rule)?;

        Ok(CddlType::CborType(CborType::MajorType7(val)))
    } else if rule.is_any() {
        Ok(CddlType::CborType(CborType::Any))
    } else {
        // TODO: after complete parsing all possible variations of `type2` rule need to drop
        // an error instead this
        Ok(CddlType::Choice(vec![]))
    }
}
