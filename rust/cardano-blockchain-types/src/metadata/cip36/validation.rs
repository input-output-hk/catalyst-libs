//! Validation function for CIP-36
//!
//! The validation include the following:
//! * Signature validation of the registration witness 61285 against the stake public key
//!   in key registration 61284.
//! * Payment address network validation against the network. The given network should
//!   match the network tag within the payment address.
//! * Purpose validation, the purpose should be 0 for Catalyst (when `is_strict_catalyst`
//!   is true).
//! * Voting keys validation, Catalyst supports only a single voting key per registration
//!   when `is_strict_catalyst` is true.

use super::Cip36;
use crate::{MetadatumValue, Network};

/// Project Catalyst Purpose
pub const PROJECT_CATALYST_PURPOSE: u64 = 0;

/// Signdata Preamble = `{ 61284: ?? }`
/// CBOR Decoded =
/// A1       # map(1)
/// 19 EF64  # unsigned(61284)
pub const SIGNDATA_PREAMBLE: [u8; 4] = [0xA1, 0x19, 0xEF, 0x64];

impl Cip36 {
    /// Validate the signature against the public key.
    pub(crate) fn validate_signature(
        &mut self,
        metadata: &MetadatumValue,
    ) {
        let hash = blake2b_simd::Params::new()
            .hash_length(32)
            .to_state()
            .update(&SIGNDATA_PREAMBLE)
            .update(metadata.as_ref())
            .finalize();

        // Ensure the signature exists
        let Some(sig) = self.registration_witness.signature else {
            self.err_report
                .missing_field("Signature", "Validate CIP36 Signature, signature not found");
            self.is_valid_signature = false;
            return;
        };

        // Ensure the stake public key exists
        let Some(stake_pk) = self.key_registration.stake_pk else {
            self.err_report.missing_field(
                "Stake public key",
                "Validate CIP36 Signature, stake public key not found",
            );
            self.is_valid_signature = false;
            return;
        };

        // Verify the signature
        if let Ok(()) = stake_pk.verify_strict(hash.as_bytes(), &sig) {
            self.is_valid_signature = true;
        } else {
            self.err_report.other(
                "Cannot verify the signature using this stake public key",
                "Validate CIP36 Signature",
            );
            self.is_valid_signature = false;
        }
    }

    /// Validate the payment address network against the given network.
    pub(crate) fn validate_payment_address_network(&mut self) {
        // Ensure the payment address exists
        let Some(address) = &self.key_registration.payment_addr else {
            self.err_report.missing_field(
                "Payment address",
                "Validate CIP36 payment address network, payment address not found",
            );
            self.is_valid_payment_address_network = false;
            return;
        };
        // Extract the network tag and validate
        let Some(network_tag) = address.network() else {
            // Byron address don't have network tag
            self.err_report.missing_field(
                "Network tag",
                "Validate CIP36 payment address network, network tag not found",
            );
            self.is_valid_payment_address_network = false;
            return;
        };

        let valid = match &self.network {
            Network::Mainnet => network_tag.value() == 1,
            Network::Preprod | Network::Preview | Network::Devnet { .. } => {
                network_tag.value() == 0
            },
        };

        // Report invalid network tag if necessary
        if !valid {
            self.err_report.invalid_value(
            "Network tag of payment address",
            &format!("{network_tag:?}"),
            &format!("Expected {}", self.network),
            "Validate CIP36 payment address network, CIP36 payment address network does not match the network used",
        );
        }

        self.is_valid_payment_address_network = valid;
    }

    /// Validate the voting keys.
    pub(crate) fn validate_voting_keys(&mut self) {
        if self.is_catalyst_strict && self.key_registration.voting_pks.len() != 1 {
            self.err_report.invalid_value(
                "Voting keys",
                &self.key_registration.voting_pks.len().to_string(),
                "Catalyst supports only a single voting key per registration",
                "Validate CIP-36 Voting Keys",
            );
            self.is_valid_voting_keys = false;
            return;
        }

        self.is_valid_voting_keys = true;
    }

    /// Validate the purpose.
    pub(crate) fn validate_purpose(&mut self) {
        if self.is_catalyst_strict && self.key_registration.purpose != PROJECT_CATALYST_PURPOSE {
            self.err_report.invalid_value(
                "Purpose",
                &self.key_registration.purpose.to_string(),
                &format!(
                    "Registration contains unknown purpose, expected {PROJECT_CATALYST_PURPOSE}"
                ),
                "Validate CIP-36 Purpose",
            );
            self.is_valid_purpose = false;
            return;
        }

        self.is_valid_purpose = true;
    }
}

#[cfg(test)]
mod tests {

    use catalyst_types::problem_report::ProblemReport;
    use ed25519_dalek::VerifyingKey;
    use pallas_addresses::Address;

    use crate::{
        Cip36, Network,
        metadata::cip36::{
            Cip36RegistrationWitness, key_registration::Cip36KeyRegistration,
            voting_pk::VotingPubKey,
        },
    };

    fn create_cip36() -> Cip36 {
        Cip36 {
            key_registration: Cip36KeyRegistration::default(),
            registration_witness: Cip36RegistrationWitness::default(),
            network: Network::Preprod,
            slot: 0.into(),
            txn_idx: 0.into(),
            is_catalyst_strict: true,
            is_valid_signature: false,
            is_valid_payment_address_network: false,
            is_valid_voting_keys: false,
            is_valid_purpose: false,
            err_report: ProblemReport::new("CIP36 Registration Validation"),
        }
    }

    #[test]
    fn test_validate_payment_address_network() {
        // cSpell:disable
        let addr = Address::from_bech32("addr_test1qprhw4s70k0vzyhvxp6h97hvrtlkrlcvlmtgmaxdtjz87xrjkctk27ypuv9dzlzxusqse89naweygpjn5dxnygvus05sdq9h07").expect("Failed to create address");
        // cSpell:enable
        let mut cip36 = create_cip36();
        cip36.key_registration = Cip36KeyRegistration {
            payment_addr: Some(addr),
            ..Default::default()
        };
        cip36.validate_payment_address_network();

        assert!(!cip36.err_report.is_problematic());
        assert!(cip36.is_valid_payment_address_network);
    }

    #[test]
    fn test_validate_invalid_payment_address_network() {
        // cSpell:disable
        let addr = Address::from_bech32("addr_test1qprhw4s70k0vzyhvxp6h97hvrtlkrlcvlmtgmaxdtjz87xrjkctk27ypuv9dzlzxusqse89naweygpjn5dxnygvus05sdq9h07").expect("Failed to create address");
        // cSpell:enable
        let mut cip36 = create_cip36();
        cip36.network = Network::Mainnet;
        cip36.key_registration = Cip36KeyRegistration {
            payment_addr: Some(addr),
            ..Default::default()
        };
        cip36.validate_payment_address_network();

        assert!(cip36.err_report.is_problematic());
        assert!(!cip36.is_valid_payment_address_network);
    }

    #[test]
    fn test_validate_voting_keys() {
        let mut cip36 = create_cip36();
        cip36
            .key_registration
            .voting_pks
            .push(VotingPubKey::new(Some(VerifyingKey::default()), 1));

        cip36.validate_voting_keys();
        assert!(!cip36.err_report.is_problematic());
        assert!(cip36.is_valid_voting_keys);
    }

    #[test]
    fn test_validate_invalid_voting_keys() {
        let mut cip36 = create_cip36();
        cip36
            .key_registration
            .voting_pks
            .push(VotingPubKey::new(Some(VerifyingKey::default()), 1));
        cip36
            .key_registration
            .voting_pks
            .push(VotingPubKey::new(Some(VerifyingKey::default()), 1));

        cip36.validate_voting_keys();
        assert!(cip36.err_report.is_problematic());
        assert!(!cip36.is_valid_voting_keys);
    }

    #[test]
    fn test_validate_purpose() {
        let mut cip36 = create_cip36();
        cip36.validate_purpose();
        assert!(!cip36.err_report.is_problematic());
        assert_eq!(cip36.key_registration.purpose, 0);
        assert!(cip36.is_valid_purpose);
    }

    #[test]
    fn test_validate_invalid_purpose() {
        let mut cip36 = create_cip36();
        cip36.key_registration = Cip36KeyRegistration {
            purpose: 1,
            ..Default::default()
        };
        cip36.validate_purpose();

        assert!(cip36.err_report.is_problematic());
        assert_eq!(cip36.key_registration.purpose, 1);
        assert!(!cip36.is_valid_purpose);
    }
}
