//! Validation function for CIP-36

use super::Cip36;
use crate::{MetadatumValue, Network};

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
    cip36: &Cip36, metadata: &MetadatumValue, validation_report: &mut Vec<String>,
) -> bool {
    let hash = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(&SIGNDATA_PREAMBLE)
        .update(metadata.as_ref())
        .finalize();

    let Some(sig) = cip36.signature() else {
        validation_report.push("Validate CIP36 Signature, signature is invalid".to_string());
        return false;
    };

    if let Ok(()) = cip36.stake_pk().verify_strict(hash.as_bytes(), &sig) {
        true
    } else {
        validation_report.push("Validate CIP36 Signature, cannot verify signature".to_string());
        false
    }
}

/// Validate the payment address network against the given network.
pub(crate) fn validate_payment_address_network(
    cip36: &Cip36, network: Network, validation_report: &mut Vec<String>,
) -> Option<bool> {
    if let Some(address) = cip36.payment_address() {
        let network_tag = address.network();
        let valid = match network {
            Network::Mainnet => network_tag.value() == 1,
            Network::Preprod | Network::Preview => network_tag.value() == 0,
        };
        if !valid {
            validation_report.push(format!(
                "Validate CIP36 payment address network, network Tag of payment address {network_tag:?} does not match the network used",
            ));
        }

        Some(valid)
    } else {
        None
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

#[cfg(test)]
mod tests {

    use ed25519_dalek::VerifyingKey;
    use pallas::ledger::addresses::Address;

    use super::validate_purpose;
    use crate::{
        cip36::{
            key_registration::Cip36KeyRegistration, registration_witness::Cip36RegistrationWitness,
            validate_payment_address_network, validate_voting_keys, voting_pk::VotingPubKey,
        },
        Cip36, Network,
    };

    fn create_empty_cip36(strict: bool) -> Cip36 {
        Cip36 {
            key_registration: Cip36KeyRegistration::default(),
            registration_witness: Cip36RegistrationWitness::default(),
            is_catalyst_strict: strict,
        }
    }

    #[test]
    fn test_validate_payment_address_network() {
        let mut cip36 = create_empty_cip36(true);
        // cSpell:disable
        let addr = Address::from_bech32("addr_test1qprhw4s70k0vzyhvxp6h97hvrtlkrlcvlmtgmaxdtjz87xrjkctk27ypuv9dzlzxusqse89naweygpjn5dxnygvus05sdq9h07").expect("Failed to create address");
        // cSpell:enable
        let Address::Shelley(shelley_addr) = addr else {
            panic!("Invalid address type")
        };
        cip36.key_registration.payment_addr = Some(shelley_addr);
        let mut report = Vec::new();

        let valid = validate_payment_address_network(&cip36, Network::Preprod, &mut report);

        assert_eq!(report.len(), 0);
        assert_eq!(valid, Some(true));
    }

    #[test]
    fn test_validate_invalid_payment_address_network() {
        let mut cip36 = create_empty_cip36(true);
        // cSpell:disable
        let addr = Address::from_bech32("addr_test1qprhw4s70k0vzyhvxp6h97hvrtlkrlcvlmtgmaxdtjz87xrjkctk27ypuv9dzlzxusqse89naweygpjn5dxnygvus05sdq9h07").expect("Failed to create address");
        // cSpell:enable
        let Address::Shelley(shelley_addr) = addr else {
            panic!("Invalid address type")
        };
        cip36.key_registration.payment_addr = Some(shelley_addr);
        let mut report = Vec::new();

        let valid = validate_payment_address_network(&cip36, Network::Mainnet, &mut report);

        assert_eq!(report.len(), 1);
        assert!(report
            .first()
            .expect("Failed to get the first index")
            .contains("does not match the network used"));
        assert_eq!(valid, Some(false));
    }

    #[test]
    fn test_validate_voting_keys() {
        let mut cip36 = create_empty_cip36(true);
        cip36.key_registration.voting_pks.push(VotingPubKey {
            voting_pk: Some(VerifyingKey::default()),
            weight: 1,
        });
        let mut report = Vec::new();

        let valid = validate_voting_keys(&cip36, &mut report);

        assert_eq!(report.len(), 0);
        assert!(valid);
    }

    #[test]
    fn test_validate_invalid_voting_keys() {
        let mut cip36 = create_empty_cip36(true);
        cip36.key_registration.voting_pks.push(VotingPubKey {
            voting_pk: Some(VerifyingKey::default()),
            weight: 1,
        });
        cip36.key_registration.voting_pks.push(VotingPubKey {
            voting_pk: Some(VerifyingKey::default()),
            weight: 1,
        });
        let mut report = Vec::new();

        let valid = validate_voting_keys(&cip36, &mut report);

        assert_eq!(report.len(), 1);
        assert!(report
            .first()
            .expect("Failed to get the first index")
            .contains("Catalyst supports only a single voting key"));
        assert!(!valid);
    }

    #[test]
    fn test_validate_purpose() {
        let cip36 = create_empty_cip36(true);
        let mut report = Vec::new();

        let valid = validate_purpose(&cip36, &mut report);

        assert_eq!(report.len(), 0);
        assert_eq!(cip36.purpose(), 0);
        assert!(valid);
    }

    #[test]
    fn test_validate_invalid_purpose() {
        let mut cip36 = create_empty_cip36(true);
        cip36.key_registration.purpose = 1;
        let mut report = Vec::new();

        let valid = validate_purpose(&cip36, &mut report);

        assert_eq!(report.len(), 1);
        assert!(report
            .first()
            .expect("Failed to get the first index")
            .contains("unknown purpose"));
        assert_eq!(cip36.purpose(), 1);
        assert!(!valid);
    }
}
