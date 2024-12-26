//! CIP-36 Catalyst registration module

pub mod key_registration;
pub mod registration_witness;
mod validation;

use ed25519_dalek::VerifyingKey;
use key_registration::{Cip36KeyRegistration, VotingPubKey};
use pallas::ledger::addresses::ShelleyAddress;
use registration_witness::Cip36RegistrationWitness;
use validation::{validate_payment_address_network, validate_signature, validate_voting_keys};

use crate::{MetadatumValue, Network};

/// CIP-36 Catalyst registration
#[derive(Clone)]
pub struct Cip36 {
    /// Key registration - 61284
    pub key_registration: Cip36KeyRegistration,
    /// Registration witness - 61285
    pub registration_witness: Cip36RegistrationWitness,
    /// Is this a Catalyst strict registration?
    pub is_catalyst_strict: bool,
}

/// CIP36 Validation Report
#[derive(Clone)]
pub struct Cip36Validation {
    /// Is the signature valid? (signature in 61285)
    pub is_valid_signature: bool,
    /// Is the payment address on the correct network?
    pub is_valid_payment_address_network: bool,
    /// Is the voting keys valid?
    pub is_valid_voting_keys: bool,
    /// Is the purpose valid? (Always 0 for Catalyst)
    pub is_valid_purpose: bool,
}

impl Cip36 {
    pub fn is_cip36(&self) -> Option<bool> {
        self.key_registration.is_cip36
    }

    pub fn voting_pks(&self) -> Vec<VotingPubKey> {
        self.key_registration.voting_pks.clone()
    }

    pub fn stake_pk(&self) -> VerifyingKey {
        self.key_registration.stake_pk.clone()
    }

    pub fn payment_address(&self) -> Option<ShelleyAddress> {
        self.key_registration.payment_addr.clone()
    }

    pub fn nonce(&self) -> u64 {
        self.key_registration.nonce
    }

    pub fn purpose(&self) -> u64 {
        self.key_registration.purpose
    }

    pub fn raw_nonce(&self) -> u64 {
        self.key_registration.raw_nonce
    }

    pub fn signature(&self) -> ed25519_dalek::Signature {
        self.registration_witness.signature.clone()
    }
    pub fn is_strict_catalyst(&self) -> bool {
        self.is_catalyst_strict
    }

    pub fn validate(
        &self, network: Network, metadata: &MetadatumValue, validation_report: &mut Vec<String>,
    ) -> Cip36Validation {
        let is_valid_signature = validate_signature(self, metadata, validation_report);
        let is_valid_payment_address_network =
            validate_payment_address_network(&self, network, validation_report)
                .unwrap_or_default();
        let is_valid_purpose = validation::validate_purpose(self, validation_report);
        let is_valid_voting_keys = validate_voting_keys(self, validation_report);

        Cip36Validation {
            is_valid_signature,
            is_valid_payment_address_network,
            is_valid_purpose,
            is_valid_voting_keys,
        }
    }
}
