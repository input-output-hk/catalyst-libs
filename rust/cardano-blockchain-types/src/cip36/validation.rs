use std::sync::Arc;
use crate::Network;
use super::Cip36;

/// Project Catalyst Purpose
pub const PROJECT_CATALYST_PURPOSE: u64 = 0;

/// Signdata Preamble = `{ 61284: ?? }`
/// CBOR Decoded =
/// A1       # map(1)
/// 19 EF64  # unsigned(61284)
pub const SIGNDATA_PREAMBLE: [u8; 4] = [0xA1, 0x19, 0xEF, 0x64];

/// Validate the signature against the public key.
#[allow(clippy::too_many_lines)]
pub(crate) fn validate_signature(
    cip36: &Cip36, metadata: &Arc<Vec<u8>>, validation_report: &mut Vec<String>,
) -> bool {
    let hash = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(&SIGNDATA_PREAMBLE)
        .update(metadata)
        .finalize();

    match cip36
        .stake_pk()
        .verify_strict(hash.as_bytes(), &cip36.signature())
    {
        Ok(_) => true,
        Err(_) => {
            validation_report.push(format!("Validate CIP36 Signature, signature is invalid"));
            false
        },
    }
}

/// Validate the payment address network against the given network.
pub(crate) fn validate_payment_address_network(
    cip36: &Cip36, network: Network, validation_report: &mut Vec<String>,
) -> Option<bool> {
    if let Some(address) = cip36.payment_address() {
        let network_tag = address.typeid();
        let valid = match network {
            Network::Mainnet => network_tag == 1,
            Network::Preprod | Network::Preview => network_tag == 0,
        };
        if !valid {
            validation_report.push(format!(
                "Validate CIP36 payment address network, network Tag {network_tag} does not match the network used"
            ));
        }

        Some(valid)
    } else {
        return None;
    }
}

/// Validate the voting keys.
pub(crate) fn validate_voting_keys(cip36: &Cip36, validation_report: &mut Vec<String>) -> bool {
    if cip36.is_strict_catalyst() && cip36.voting_pks().len() != 1 {
        validation_report.push(format!(
            "Validate CIP-36 Voting Keys, Catalyst supports only a single voting key per registration, found {}",
            cip36.voting_pks().len()
        ));
        return false;
    }
    true
}

/// Validate the purpose.
pub(crate) fn validate_purpose(cip36: &Cip36, validation_report: &mut Vec<String>) -> bool {
    if cip36.is_strict_catalyst() && cip36.purpose() != PROJECT_CATALYST_PURPOSE {
        validation_report.push(format!(
            "Validate CIP-36 Purpose, registration contains unknown purpose: {}",
            cip36.purpose()
        ));
        return false;
    }
    true
}
