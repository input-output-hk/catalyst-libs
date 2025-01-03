//! CIP-36 Catalyst registration module

pub mod key_registration;
pub mod registration_witness;
mod validation;
pub mod voting_pk;

use anyhow::bail;
use ed25519_dalek::VerifyingKey;
use key_registration::Cip36KeyRegistration;
use minicbor::{Decode, Decoder};
use pallas::ledger::addresses::ShelleyAddress;
use registration_witness::Cip36RegistrationWitness;
use validation::{validate_payment_address_network, validate_signature, validate_voting_keys};
use voting_pk::VotingPubKey;

use crate::{MetadatumLabel, MetadatumValue, MultiEraBlock, Network, TxnIndex};

/// CIP-36 Catalyst registration
#[derive(Clone, Default, Debug)]
pub struct Cip36 {
    /// Key registration - 61284
    key_registration: Cip36KeyRegistration,
    /// Registration witness - 61285
    registration_witness: Cip36RegistrationWitness,
    /// Is this a Catalyst strict registration?
    is_catalyst_strict: bool,
}

/// Validation value for CIP-36.
#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug)]
pub(crate) struct Cip36Validation {
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
    /// Create an instance of CIP-36.
    /// The CIP-36 registration contains the key registration (61284)
    /// and registration witness (61285) metadata.
    ///
    /// # Parameters
    ///
    /// * `block` - The block containing the auxiliary data.
    /// * `txn_idx` - The transaction index that contain the auxiliary data.
    /// * `is_catalyst_strict` - Is this a Catalyst strict registration?
    ///
    /// # Errors
    ///
    /// If the CIP-36 key registration or registration witness metadata is not found.
    /// or if the CIP-36 key registration or registration witness metadata cannot be
    /// decoded.
    pub fn new(
        block: &MultiEraBlock, txn_idx: TxnIndex, is_catalyst_strict: bool,
    ) -> anyhow::Result<Self> {
        let Some(k61284) = block.txn_metadata(txn_idx, MetadatumLabel::CIP036_REGISTRATION) else {
            bail!("CIP-36 key registration metadata not found")
        };
        let Some(k61285) = block.txn_metadata(txn_idx, MetadatumLabel::CIP036_WITNESS) else {
            bail!("CIP-36 registration witness metadata not found")
        };

        let slot = block.decode().slot();
        let network = block.network();

        let mut key_registration = Decoder::new(k61284.as_ref());
        let mut registration_witness = Decoder::new(k61285.as_ref());

        let key_registration = match Cip36KeyRegistration::decode(&mut key_registration, &mut ()) {
            Ok(mut metadata) => {
                let nonce = if is_catalyst_strict && metadata.raw_nonce > Some(slot) {
                    Some(slot)
                } else {
                    metadata.raw_nonce
                };

                metadata.nonce = nonce;
                metadata
            },
            Err(e) => {
                bail!("Failed to construct CIP-36 key registration, {e}")
            },
        };

        let registration_witness =
            match Cip36RegistrationWitness::decode(&mut registration_witness, &mut ()) {
                Ok(metadata) => metadata,
                Err(e) => {
                    bail!("Failed to construct CIP-36 registration witness {e}")
                },
            };

        let cip36 = Self {
            key_registration,
            registration_witness,
            is_catalyst_strict,
        };

        let mut validation_report = Vec::new();
        // If the code reach here, then the CIP36 decoding is successful.
        let validation = cip36.validate(network, k61284, &mut validation_report);

        if validation.is_valid_signature
            && validation.is_valid_payment_address_network
            && validation.is_valid_voting_keys
            && validation.is_valid_purpose
        {
            Ok(cip36)
        } else {
            bail!("CIP-36 validation failed: {validation:?}, Reports: {validation_report:?}")
        }
    }

    /// Get the `is_cip36` flag from the registration.
    /// True if it is CIP-36 format, false if CIP-15 format.
    #[must_use]
    pub fn is_cip36(&self) -> Option<bool> {
        self.key_registration.is_cip36
    }

    /// Get the voting public keys from the registration.
    #[must_use]
    pub fn voting_pks(&self) -> &Vec<VotingPubKey> {
        &self.key_registration.voting_pks
    }

    /// Get the stake public key from the registration.
    #[must_use]
    pub fn stake_pk(&self) -> Option<&VerifyingKey> {
        self.key_registration.stake_pk.as_ref()
    }

    /// Get the payment address from the registration.
    #[must_use]
    pub fn payment_address(&self) -> Option<&ShelleyAddress> {
        self.key_registration.payment_addr.as_ref()
    }

    /// Get the nonce from the registration.
    #[must_use]
    pub fn nonce(&self) -> Option<u64> {
        self.key_registration.nonce
    }

    /// Get the purpose from the registration.
    #[must_use]
    pub fn purpose(&self) -> Option<u64> {
        self.key_registration.purpose
    }

    /// Get the raw nonce from the registration.
    #[must_use]
    pub fn raw_nonce(&self) -> Option<u64> {
        self.key_registration.raw_nonce
    }

    /// Is the payment address in the registration payable?
    #[must_use]
    pub fn is_payable(&self) -> Option<bool> {
        self.key_registration.is_payable
    }

    /// Get the signature from the registration witness.
    #[must_use]
    pub fn signature(&self) -> Option<ed25519_dalek::Signature> {
        self.registration_witness.signature
    }

    /// Get the Catalyst strict flag.
    #[must_use]
    pub fn is_strict_catalyst(&self) -> bool {
        self.is_catalyst_strict
    }

    /// Validation for CIP-36
    /// The validation include the following:
    /// * Signature validation of the registration witness 61285 against the stake public
    ///   key in key registration 61284.
    /// * Payment address network validation against the network. The given network should
    ///   match the network tag within the payment address.
    /// * Purpose validation, the purpose should be 0 for Catalyst (when
    ///   `is_strict_catalyst` is true).
    /// * Voting keys validation, Catalyst supports only a single voting key per
    ///   registration when `is_strict_catalyst` is true.
    ///
    /// # Parameters
    ///
    /// * `network` - The blockchain network.
    /// * `metadata` - The metadata value to be validated.
    /// * `validation_report` - Validation report to store the validation result.
    fn validate(
        &self, network: Network, metadata: &MetadatumValue, validation_report: &mut Vec<String>,
    ) -> Cip36Validation {
        let is_valid_signature = validate_signature(self, metadata, validation_report);
        let is_valid_payment_address_network =
            validate_payment_address_network(self, network, validation_report).unwrap_or_default();
        let is_valid_voting_keys = validate_voting_keys(self, validation_report);
        let is_valid_purpose = validation::validate_purpose(self, validation_report);

        Cip36Validation {
            is_valid_signature,
            is_valid_payment_address_network,
            is_valid_voting_keys,
            is_valid_purpose,
        }
    }
}
