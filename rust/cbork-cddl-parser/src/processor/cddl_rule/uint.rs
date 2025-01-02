//! Process `uint` rule

use anyhow::anyhow;

use super::CddlRule;

/// Process `uint` rule
pub(crate) fn process_uint_rule(uint_rule: &impl CddlRule) -> anyhow::Result<u64> {
    let uint_rule_str = uint_rule.to_string();

    if let Some(hex_str) = uint_rule_str.strip_prefix("0x") {
        u64::from_str_radix(hex_str, 16)
            .map_err(|e| anyhow!("Invalid `uint` rule, cannot parse hex value, err: {e}"))
    } else if let Some(bin_str) = uint_rule_str.strip_prefix("0b") {
        u64::from_str_radix(bin_str, 2)
            .map_err(|e| anyhow!("Invalid `uint` rule, cannot parse bin value, err: {e}"))
    } else {
        uint_rule_str
            .parse()
            .map_err(|e| anyhow!("Invalid `uint` rule, cannot parse value, err: {e}"))
    }
}
