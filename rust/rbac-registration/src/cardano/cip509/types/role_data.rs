//! RBAC role data

use std::collections::HashMap;

use pallas::ledger::{addresses::ShelleyAddress, primitives::conway};

use crate::cardano::cip509::{rbac::role_data::CborRoleData, KeyLocalRef};

/// A role data.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleData {
    /// A signing key.
    signing_key: Option<KeyLocalRef>,
    /// An encryption key.
    encryption_key: Option<KeyLocalRef>,
    /// A payment key where reward will be distributed to.
    payment_key: Option<ShelleyAddress>,
    /// A map of the extended data.
    extended_data: HashMap<u8, Vec<u8>>,
}

impl RoleData {
    /// Create an instance of role data.
    pub fn new(data: CborRoleData, transaction: &conway::MintedTx) -> Self {
        let payment_key = todo!();

        Self {
            signing_key: data.signing_key,
            encryption_key: data.encryption_key,
            payment_key,
            extended_data: data.extended_data,
        }
    }

    /// Returns a reference to the signing key.
    #[must_use]
    pub fn signing_key(&self) -> Option<&KeyLocalRef> {
        self.signing_key.as_ref()
    }

    /// Returns a reference to the encryption key.
    #[must_use]
    pub fn encryption_key(&self) -> Option<&KeyLocalRef> {
        self.encryption_key.as_ref()
    }

    /// Returns a reference to the payment key.
    #[must_use]
    pub fn payment_key(&self) -> Option<&ShelleyAddress> {
        self.payment_key.as_ref()
    }

    /// Returns a reference to the extended data.
    #[must_use]
    pub fn extended_data(&self) -> &HashMap<u8, Vec<u8>> {
        &self.extended_data
    }

    /// Sets a new value for the signing key.
    pub fn set_signing_key(&mut self, key: Option<KeyLocalRef>) {
        self.signing_key = key;
    }

    /// Sets a new value for the encryption key.
    pub fn set_encryption_key(&mut self, key: Option<KeyLocalRef>) {
        self.encryption_key = key;
    }
}

// TODO: FIXME:
/// Helper function for retrieving the Shelley address from the transaction.
fn get_payment_addr_from_tx(
    txn: &conway::MintedTx, payment_key_ref: Option<i16>,
) -> anyhow::Result<Option<ShelleyAddress>> {
    // The index should exist since it pass the basic validation
    if let Some(key_ref) = payment_key_ref {
        // Transaction output
        if key_ref < 0 {
            let index = decremented_index(key_ref.abs())?;
            if let Some(output) = tx.transaction_body.outputs.get(index) {
                // Conway era -> Post alonzo tx output
                match output {
                    conway::PseudoTransactionOutput::PostAlonzo(o) => {
                        let address =
                            Address::from_bytes(&o.address).map_err(|e| anyhow::anyhow!(e))?;

                        if let Address::Shelley(addr) = address {
                            return Ok(Some(addr.clone()));
                        }
                        bail!("Unsupported address type in payment key reference");
                    },
                    // Not support legacy form of transaction output
                    conway::PseudoTransactionOutput::Legacy(_) => {
                        bail!("Unsupported transaction output type in payment key reference");
                    },
                }
            }
            // Index doesn't exist
            bail!("Payment key not found in transaction output");
        }
        // Transaction input, currently unsupported because of the reference to transaction hash
        bail!("Unsupported payment key reference to transaction input");
    }
    Ok(None)
}

//

// TODO: FIXME:
/// Checks that the payment key reference is correct and points to a valid key.
pub fn validate_payment_key(
    transaction: &MultiEraTx, conway_transaction: &conway::MintedTx, role_data: &RoleData,
    report: &ProblemReport,
) {
    let context = "Cip509 role0 payment key validation";

    let Some(payment_key) = role_data.payment_key() else {
        report.other("Missing payment key in role0", context);
        return;
    };
    if payment_key == 0 {
        report.invalid_value(
            "payment key",
            "0",
            "Payment reference key must not be 0",
            context,
        );
        return;
    }

    // Negative indicates reference to transaction output.
    if payment_key < 0 {
        let index = match decremented_index(payment_key.abs()) {
            Ok(value) => value,
            Err(e) => {
                report.other(
                    &format!("Failed to get index of payment key: {e:?}"),
                    context,
                );
                return;
            },
        };
        let outputs = &conway_transaction.transaction_body.outputs;
        let witness = match TxWitness::new(&[transaction.clone()]) {
            Ok(witnesses) => witnesses,
            Err(e) => {
                report.other(&format!("Failed to create TxWitness: {e:?}"), context);
                return;
            },
        };

        let address = match outputs.get(index) {
            Some(conway::PseudoTransactionOutput::Legacy(o)) => &o.address,
            Some(conway::PseudoTransactionOutput::PostAlonzo(o)) => &o.address,
            None => {
                report.other(
                    &format!(
                        "Role payment key reference index ({index}) is not found
in transaction outputs"
                    ),
                    context,
                );
                return;
            },
        };
        validate_payment_output_key_helper(address, &witness, report, context);
    } else {
        // Positive indicates reference to tx input.
        let inputs = &conway_transaction.transaction_body.inputs;
        let index = match decremented_index(payment_key) {
            Ok(value) => value,
            Err(e) => {
                report.other(
                    &format!("Failed to get index of payment key: {e:?}"),
                    context,
                );
                return;
            },
        };
        // Check whether the index exists in transaction inputs.
        if inputs.get(index).is_none() {
            report.other(
                &format!(
                    "Role payment key reference index ({index}) is not found in
transaction inputs"
                ),
                context,
            );
        }
    }
}

/// Helper function for validating payment output key.
fn validate_payment_output_key_helper(
    output_address: &[u8], witness: &TxWitness, report: &ProblemReport, context: &str,
) {
    let Some(key) = extract_key_hash(output_address) else {
        report.other("Failed to extract payment key hash from address", context);
        return;
    };

    // Set transaction index to 0 because the list of transaction is manually constructed
    // for TxWitness -> &[txn.clone()], so we can assume that the witness contains only
    // the witness within this transaction.
    if let Err(e) = compare_key_hash(&[key.clone()], witness, 0) {
        report.other(
            &format!(
                "Unable to find payment output key ({key:?}) in the transaction witness
set: {e:?}"
            ),
            context,
        );
    }
}
