//! CIP-36 Catalyst registration module

pub mod key_registration;
pub mod registration_witness;
mod validation;
pub mod voting_pk;

use anyhow::bail;
use catalyst_types::problem_report::ProblemReport;
use ed25519_dalek::VerifyingKey;
use key_registration::Cip36KeyRegistration;
use minicbor::{Decode, Decoder};
use pallas::ledger::addresses::ShelleyAddress;
use registration_witness::Cip36RegistrationWitness;
use validation::{validate_cip36, Cip36Validation};
use voting_pk::VotingPubKey;

use crate::{MetadatumLabel, MultiEraBlock, TxnIndex};

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
    /// # Returns
    /// A tuple containing the CIP-36 registration, the validation result, and a problem report.
    /// 
    /// # Errors
    ///
    /// If the CIP-36 key registration or registration witness metadata is not found.
    /// or if the CIP-36 key registration or registration witness metadata cannot be
    /// decoded.
    pub fn new(
        block: &MultiEraBlock, txn_idx: TxnIndex, is_catalyst_strict: bool,
    ) -> anyhow::Result<(Cip36, Cip36Validation, ProblemReport)> {
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

        // Record of errors found during decoding and validation
        let mut err_report = ProblemReport::new("CIP36 Registration Decoding and Validation");

        let key_registration =
            match Cip36KeyRegistration::decode(&mut key_registration, &mut err_report) {
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
            match Cip36RegistrationWitness::decode(&mut registration_witness, &mut err_report) {
                Ok(metadata) => metadata,
                Err(e) => {
                    bail!("Failed to construct CIP-36 registration witness {e}")
                },
            };

        // If the code reach here, then the CIP36 decoding is successful.
        // Now check whether everything is valid.
        let validation = validate_cip36(
            &key_registration,
            &registration_witness,
            is_catalyst_strict,
            network,
            k61284,
            &err_report,
        );

        Ok((
            Self {
                key_registration,
                registration_witness,
                is_catalyst_strict,
            },
            validation,
            err_report,
        ))
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
    pub fn purpose(&self) -> u64 {
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
}
