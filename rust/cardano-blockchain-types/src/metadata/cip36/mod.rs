//! CIP-36 Catalyst registration module

pub mod key_registration;
pub mod registration_witness;
mod validation;
pub mod voting_pk;
use std::{collections::HashMap, fmt};

use catalyst_types::problem_report::ProblemReport;
use ed25519_dalek::VerifyingKey;
use key_registration::Cip36KeyRegistration;
use minicbor::{Decode, Decoder};
use pallas_addresses::Address;
use registration_witness::Cip36RegistrationWitness;
use voting_pk::VotingPubKey;

use crate::{MetadatumLabel, MultiEraBlock, Network, Slot, TxnIndex};

/// CIP-36 Catalyst registration
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cip36 {
    /// Key registration - 61284
    key_registration: Cip36KeyRegistration,
    /// Registration witness - 61285
    registration_witness: Cip36RegistrationWitness,
    /// Network that this CIP-36 registration is on.
    network: Network,
    /// Slot that this CIP-36 registration is on.
    slot: Slot,
    /// Transaction index that this CIP-36 registration is on.
    txn_idx: TxnIndex,
    /// Is this a Catalyst strict registration?
    is_catalyst_strict: bool,
    /// Is the signature valid? (signature in 61285)
    is_valid_signature: bool,
    /// Is the payment address on the correct network?
    is_valid_payment_address_network: bool,
    /// Is the voting keys valid?
    is_valid_voting_keys: bool,
    /// Is the purpose valid? (Always 0 for Catalyst)
    is_valid_purpose: bool,
    /// Error report.
    err_report: ProblemReport,
}

impl fmt::Display for Cip36 {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "Cip36 {{ network: {network}, slot: {slot:?}, txn_idx: {txn_idx:?}, is_catalyst_strict: {is_catalyst_strict}, key_registration: {key_registration:?}, registration_witness: {registration_witness:?}, validation: {{ signature: {is_valid_signature}, payment_address_network: {is_valid_payment_address_network}, voting_keys: {is_valid_voting_keys}, purpose: {is_valid_purpose} }}, err_report: {err_report} }}",
            key_registration = self.key_registration,
            registration_witness = self.registration_witness,
            network = self.network,
            slot = self.slot,
            txn_idx = self.txn_idx,
            is_catalyst_strict = self.is_catalyst_strict,
            is_valid_signature = self.is_valid_signature,
            is_valid_payment_address_network = self.is_valid_payment_address_network,
            is_valid_voting_keys = self.is_valid_voting_keys,
            is_valid_purpose = self.is_valid_purpose,
            err_report = serde_json::to_string(&self.err_report)
            .unwrap_or_else(|_| String::from("Failed to serialize ProblemReport"))
        )
    }
}

/// CIP-36 Catalyst registration error
#[allow(dead_code, clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct Cip36Error {
    /// The decoding error that make the code not able to process.
    error: anyhow::Error,
    /// The problem report that contains the errors found during decoding and validation.
    report: ProblemReport,
}

impl fmt::Display for Cip36Error {
    fn fmt(
        &self,
        fmt: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let report_json = serde_json::to_string(&self.report)
            .unwrap_or_else(|_| String::from("Failed to serialize ProblemReport"));

        write!(
            fmt,
            "Cip36Error {{ error: {}, report: {} }}",
            self.error, report_json
        )
    }
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
    ///
    /// None if the metadata is not in the block at given index.
    ///
    /// # Errors
    ///
    /// If the CIP-36 key registration or registration witness metadata is not found.
    /// or if the CIP-36 key registration or registration witness metadata cannot be
    /// decoded.
    pub fn new(
        block: &MultiEraBlock,
        txn_idx: TxnIndex,
        is_catalyst_strict: bool,
    ) -> Result<Option<Cip36>, Cip36Error> {
        // Record of errors found during decoding and validation
        let mut err_report = ProblemReport::new("CIP36 Registration Decoding and Validation");

        let Some(k61284) = block.txn_metadata(txn_idx, MetadatumLabel::CIP036_REGISTRATION) else {
            return Ok(None);
        };
        let Some(k61285) = block.txn_metadata(txn_idx, MetadatumLabel::CIP036_WITNESS) else {
            return Ok(None);
        };

        let slot = block.decode().slot();
        let network = block.network();

        let mut key_registration = Decoder::new(k61284.as_ref());
        let mut registration_witness = Decoder::new(k61285.as_ref());

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
                    return Err(Cip36Error {
                        error: anyhow::anyhow!(format!(
                            "Failed to construct CIP-36 key registration, {e}"
                        )),
                        report: err_report,
                    });
                },
            };

        let registration_witness =
            match Cip36RegistrationWitness::decode(&mut registration_witness, &mut err_report) {
                Ok(metadata) => metadata,
                Err(e) => {
                    return Err(Cip36Error {
                        error: anyhow::anyhow!(format!(
                            "Failed to construct CIP-36 registration witness {e}"
                        )),
                        report: err_report,
                    });
                },
            };

        // If the code reach here, then the CIP36 decoding is successful.
        // Construct the CIP-36 registration
        let mut cip36 = Cip36 {
            key_registration,
            registration_witness,
            network,
            slot: slot.into(),
            txn_idx,
            is_catalyst_strict,
            is_valid_signature: false,
            is_valid_payment_address_network: false,
            is_valid_voting_keys: false,
            is_valid_purpose: false,
            err_report,
        };

        // Now check whether everything is valid.
        cip36.validate_signature(k61284);
        cip36.validate_payment_address_network();
        cip36.validate_voting_keys();
        cip36.validate_purpose();

        Ok(Some(cip36))
    }

    /// Collect all CIP-36 registrations from a block.
    ///
    /// # Parameters
    ///
    /// * `block` - The block that wanted to be processed.
    /// * `is_catalyst_strict` - Is this a Catalyst strict registration?
    ///
    /// # Returns
    ///
    /// A map of transaction index to the Result of CIP-36 and its errors.
    /// None if there is no CIP-36 registration found in the block.
    #[must_use]
    pub fn cip36_from_block(
        block: &MultiEraBlock,
        is_catalyst_strict: bool,
    ) -> Option<HashMap<TxnIndex, Result<Cip36, Cip36Error>>> {
        let mut cip36_map = HashMap::new();

        for (txn_idx, _tx) in block.decode().txs().iter().enumerate() {
            let txn_idx: TxnIndex = txn_idx.into();
            let cip36 = Cip36::new(block, txn_idx, is_catalyst_strict);
            match cip36 {
                Ok(Some(cip36)) => {
                    cip36_map.insert(txn_idx, Ok(cip36));
                },
                // None - no CIP-36 metadata found in the block
                Ok(None) => {},
                // Error - found CIP-36 but there is some error
                Err(e) => {
                    cip36_map.insert(txn_idx, Err(e));
                },
            }
        }

        if cip36_map.is_empty() {
            return None;
        }
        Some(cip36_map)
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
    pub fn payment_address(&self) -> Option<&Address> {
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

    /// Get the slot number of this CIP-36 registration.
    #[must_use]
    pub fn slot(&self) -> Slot {
        self.slot
    }

    /// Get the network of this CIP-36 registration.
    #[must_use]
    pub fn network(&self) -> Network {
        self.network
    }

    /// Get the transaction index of this CIP-36 registration.
    #[must_use]
    pub fn txn_idx(&self) -> TxnIndex {
        self.txn_idx
    }

    /// Get the Catalyst strict flag.
    #[must_use]
    pub fn is_strict_catalyst(&self) -> bool {
        self.is_catalyst_strict
    }

    /// Is the CIP-36 registration valid?
    #[must_use]
    pub fn is_valid(&self) -> bool {
        // Check everything
        self.is_valid_signature
            && self.is_valid_payment_address_network
            && self.is_valid_voting_keys
            && self.is_valid_purpose
            && !self.err_report.is_problematic()
    }

    /// Is the signature valid?
    #[must_use]
    pub fn is_valid_signature(&self) -> bool {
        self.is_valid_signature
    }

    /// Is the payment address network tag match the provided network?
    #[must_use]
    pub fn is_valid_payment_address_network(&self) -> bool {
        self.is_valid_payment_address_network
    }

    /// Is the voting keys valid?
    #[must_use]
    pub fn is_valid_voting_keys(&self) -> bool {
        self.is_valid_voting_keys
    }

    /// Is the purpose valid?
    #[must_use]
    pub fn is_valid_purpose(&self) -> bool {
        self.is_valid_purpose
    }

    /// Get the error report.
    #[must_use]
    pub fn err_report(&self) -> &ProblemReport {
        &self.err_report
    }
}

#[cfg(test)]
mod tests {
    use crate::{Cip36, MultiEraBlock, Network, Point};

    // CIP36 in transaction 1
    // <https://preprod.cardanoscan.io/transaction/5c7cf7bff543447fc1e46c2598380fa08b8d882de2191ef5bc80078b71724217?tab=metadata>
    fn block_1() -> MultiEraBlock {
        let data = hex::decode(include_str!("./test_data/block_1.block")).unwrap();
        let previous = Point::fuzzy(0.into());
        MultiEraBlock::new(Network::Preprod, data, &previous, 0.into()).unwrap()
    }

    #[test]
    fn new() {
        let res = Cip36::new(&block_1(), 1.into(), true).unwrap().unwrap();
        assert!(!res.err_report().is_problematic());
        assert!(res.is_valid());
        assert!(res.network() == Network::Preprod);
        assert!(res.raw_nonce() == Some(55_076_993));
        assert!(res.nonce() == Some(55_076_993));
    }

    #[test]
    fn from_block() {
        let res = Cip36::cip36_from_block(&block_1(), true).unwrap();
        assert_eq!(res.len(), 1);
    }
}
