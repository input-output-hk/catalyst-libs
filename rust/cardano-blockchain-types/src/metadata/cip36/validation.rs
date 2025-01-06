//! Validation function for CIP-36

use catalyst_types::problem_report::ProblemReport;

use super::{Cip36KeyRegistration, Cip36RegistrationWitness};
use crate::{MetadatumValue, Network};

/// Project Catalyst Purpose
pub const PROJECT_CATALYST_PURPOSE: u64 = 0;

/// Signdata Preamble = `{ 61284: ?? }`
/// CBOR Decoded =
/// A1       # map(1)
/// 19 EF64  # unsigned(61284)
pub const SIGNDATA_PREAMBLE: [u8; 4] = [0xA1, 0x19, 0xEF, 0x64];

/// Validation value for CIP-36.
#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub(crate) struct Cip36Validation {
    /// Is the signature valid? (signature in 61285)
    is_valid_signature: bool,
    /// Is the payment address on the correct network?
    is_valid_payment_address_network: bool,
    /// Is the voting keys valid?
    is_valid_voting_keys: bool,
    /// Is the purpose valid? (Always 0 for Catalyst)
    is_valid_purpose: bool,
}

/// Validation for CIP-36
/// The validation include the following:
/// * Signature validation of the registration witness 61285 against the stake public key
///   in key registration 61284.
/// * Payment address network validation against the network. The given network should
///   match the network tag within the payment address.
/// * Purpose validation, the purpose should be 0 for Catalyst (when `is_strict_catalyst`
///   is true).
/// * Voting keys validation, Catalyst supports only a single voting key per registration
///   when `is_strict_catalyst` is true.
///
/// # Parameters
///
/// * `network` - The blockchain network.
/// * `metadata` - The metadata value to be validated.
/// * `validation_report` - Validation report to store the validation result.
pub(crate) fn validate_cip36(
    key_registration: &Cip36KeyRegistration, registration_witness: &Cip36RegistrationWitness,
    is_strict_catalyst: bool, network: Network, metadata: &MetadatumValue,
    validation_report: &ProblemReport,
) -> Cip36Validation {
    // Need to make sure that when return false, the validation_report is updated.
    let is_valid_signature = validate_signature(
        key_registration,
        registration_witness,
        metadata,
        validation_report,
    );
    let is_valid_payment_address_network =
        validate_payment_address_network(key_registration, network, validation_report)
            .unwrap_or_default();
    let is_valid_voting_keys =
        validate_voting_keys(key_registration, is_strict_catalyst, validation_report);
    let is_valid_purpose =
        validate_purpose(key_registration, is_strict_catalyst, validation_report);

    Cip36Validation {
        is_valid_signature,
        is_valid_payment_address_network,
        is_valid_voting_keys,
        is_valid_purpose,
    }
}

/// Validate the signature against the public key.
#[allow(clippy::too_many_lines)]
fn validate_signature(
    key_registration: &Cip36KeyRegistration, registration_witness: &Cip36RegistrationWitness,
    metadata: &MetadatumValue, validation_report: &ProblemReport,
) -> bool {
    let hash = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(&SIGNDATA_PREAMBLE)
        .update(metadata.as_ref())
        .finalize();

    let Some(sig) = registration_witness.signature else {
        validation_report
            .missing_field("Signature", "Validate CIP36 Signature, signature not found");
        return false;
    };

    if let Some(stake_pk) = key_registration.stake_pk {
        if let Ok(()) = stake_pk.verify_strict(hash.as_bytes(), &sig) {
            return true;
        }
        validation_report.other(
            "Cannot verify the signature using stake public key",
            "Validate CIP36 Signature",
        );
        return false;
    }

    validation_report.missing_field(
        "Stake public key",
        "Validate CIP36 Signature, stake public Key not found",
    );
    false
}

/// Validate the payment address network against the given network.
fn validate_payment_address_network(
    key_registration: &Cip36KeyRegistration, network: Network, validation_report: &ProblemReport,
) -> Option<bool> {
    if let Some(address) = &key_registration.payment_addr {
        let network_tag = address.network();
        let valid = match network {
            Network::Mainnet => network_tag.value() == 1,
            Network::Preprod | Network::Preview => network_tag.value() == 0,
        };
        if !valid {
            validation_report.invalid_value(
                "Network tag of payment address",
                format!("{network_tag:?}").as_str(),
                format!("Expected {network}").as_str(),
                "Validate CIP36 payment address network, CIP36 payment address network does not match the network used",
            );
        }

        Some(valid)
    } else {
        validation_report.missing_field(
            "Payment address",
            "Validate CIP36 payment address network, payment address not found",
        );
        None
    }
}

/// Validate the voting keys.
fn validate_voting_keys(
    key_registration: &Cip36KeyRegistration, is_strict_catalyst: bool,
    validation_report: &ProblemReport,
) -> bool {
    if is_strict_catalyst && key_registration.voting_pks.len() != 1 {
        validation_report.invalid_value(
            "Voting keys",
            format!("{}", key_registration.voting_pks.len()).as_str(),
            "Catalyst supports only a single voting key per registration",
            "Validate CIP-36 Voting Keys",
        );
        return false;
    }
    true
}

/// Validate the purpose.
fn validate_purpose(
    key_registration: &Cip36KeyRegistration, is_strict_catalyst: bool,
    validation_report: &ProblemReport,
) -> bool {
    if is_strict_catalyst && key_registration.purpose != PROJECT_CATALYST_PURPOSE {
        validation_report.invalid_value(
            "Purpose",
            format!("{}", key_registration.purpose).as_str(),
            format!("Registration contains unknown purpose, expected {PROJECT_CATALYST_PURPOSE}")
                .as_str(),
            "Validate CIP-36 Purpose",
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {

    use catalyst_types::problem_report::ProblemReport;
    use ed25519_dalek::VerifyingKey;
    use pallas::ledger::addresses::Address;

    use super::validate_purpose;
    use crate::{
        metadata::cip36::{
            key_registration::Cip36KeyRegistration,
            validation::{validate_payment_address_network, validate_voting_keys},
            voting_pk::VotingPubKey,
        },
        Network,
    };

    #[test]
    fn test_validate_payment_address_network() {
        // cSpell:disable
        let addr = Address::from_bech32("addr_test1qprhw4s70k0vzyhvxp6h97hvrtlkrlcvlmtgmaxdtjz87xrjkctk27ypuv9dzlzxusqse89naweygpjn5dxnygvus05sdq9h07").expect("Failed to create address");
        // cSpell:enable
        let Address::Shelley(shelley_addr) = addr else {
            panic!("Invalid address type")
        };
        let key_registration = Cip36KeyRegistration {
            payment_addr: Some(shelley_addr),
            ..Default::default()
        };
        let validation_report = ProblemReport::new("CIP36 Registration Validation");
        let valid = validate_payment_address_network(
            &key_registration,
            Network::Preprod,
            &validation_report,
        );

        assert!(!validation_report.is_problematic());
        assert_eq!(valid, Some(true));
    }

    #[test]
    fn test_validate_invalid_payment_address_network() {
        // cSpell:disable
        let addr = Address::from_bech32("addr_test1qprhw4s70k0vzyhvxp6h97hvrtlkrlcvlmtgmaxdtjz87xrjkctk27ypuv9dzlzxusqse89naweygpjn5dxnygvus05sdq9h07").expect("Failed to create address");
        // cSpell:enable
        let Address::Shelley(shelley_addr) = addr else {
            panic!("Invalid address type")
        };
        let key_registration = Cip36KeyRegistration {
            payment_addr: Some(shelley_addr),
            ..Default::default()
        };
        let validation_report = ProblemReport::new("CIP36 Registration Validation");
        let valid = validate_payment_address_network(
            &key_registration,
            Network::Mainnet,
            &validation_report,
        );

        assert!(validation_report.is_problematic());
        assert!(serde_json::to_string(&validation_report)
            .unwrap_or_else(|_| "Failed to serialize ProblemReport".to_string())
            .contains("does not match the network used"));
        assert_eq!(valid, Some(false));
    }

    #[test]
    fn test_validate_voting_keys() {
        let mut key_registration = Cip36KeyRegistration::default();

        key_registration
            .voting_pks
            .push(VotingPubKey::new(Some(VerifyingKey::default()), 1));
        let validation_report = ProblemReport::new("CIP36 Registration Validation");

        let valid = validate_voting_keys(&key_registration, true, &validation_report);

        assert!(!validation_report.is_problematic());
        assert!(valid);
    }

    #[test]
    fn test_validate_invalid_voting_keys() {
        let mut key_registration = Cip36KeyRegistration::default();

        key_registration
            .voting_pks
            .push(VotingPubKey::new(Some(VerifyingKey::default()), 1));

        key_registration
            .voting_pks
            .push(VotingPubKey::new(Some(VerifyingKey::default()), 1));
        let validation_report = ProblemReport::new("CIP36 Registration Validation");

        let valid = validate_voting_keys(&key_registration, true, &validation_report);

        assert!(validation_report.is_problematic());
        assert!(serde_json::to_string(&validation_report)
            .unwrap_or_else(|_| "Failed to serialize ProblemReport".to_string())
            .contains("Catalyst supports only a single voting key"));
        assert!(!valid);
    }

    #[test]
    fn test_validate_purpose() {
        let key_registration = Cip36KeyRegistration::default();
        let validation_report = ProblemReport::new("CIP36 Registration Validation");

        let valid = validate_purpose(&key_registration, true, &validation_report);

        assert!(!validation_report.is_problematic());
        assert_eq!(key_registration.purpose, 0);
        assert!(valid);
    }

    #[test]
    fn test_validate_invalid_purpose() {
        let key_registration = Cip36KeyRegistration {
            purpose: 1,
            ..Default::default()
        };
        let validation_report = ProblemReport::new("CIP36 Registration Validation");

        let valid = validate_purpose(&key_registration, true, &validation_report);

        assert!(validation_report.is_problematic());
        assert!(serde_json::to_string(&validation_report)
            .unwrap_or_else(|_| "Failed to serialize ProblemReport".to_string())
            .contains("unknown purpose"));
        assert_eq!(key_registration.purpose, 1);
        assert!(!valid);
    }
}
