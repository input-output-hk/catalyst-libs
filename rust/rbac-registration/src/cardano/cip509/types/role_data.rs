//! RBAC role data

use std::{borrow::Cow, collections::HashMap};

use catalyst_types::problem_report::ProblemReport;
use pallas::ledger::{
    addresses::{Address, ShelleyAddress},
    primitives::conway,
    traverse::MultiEraTx,
};

use crate::{
    cardano::{
        cip509::{
            rbac::role_data::CborRoleData,
            utils::cip19::{compare_key_hash, extract_key_hash},
            KeyLocalRef, RoleNumber,
        },
        transaction::witness::TxWitness,
    },
    utils::general::decremented_index,
};

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
    pub fn new(data: CborRoleData, transaction: &conway::MintedTx, report: &ProblemReport) -> Self {
        let payment_key = if data.number == Some(RoleNumber::ROLE_0) && data.payment_key.is_none() {
            report.other(
                "Missing payment key in role0",
                "Role data payment key validation",
            );
            None
        } else {
            let context = format!(
                "Validating the role data payment key for {:?} role",
                data.number
            );
            convert_payment_key(data.payment_key, transaction, &context, report)
        };

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

/// Converts the payment key from the form encoded in CBOR role data to `ShelleyAddress`.
fn convert_payment_key(
    key: Option<i16>, transaction: &conway::MintedTx, context: &str, report: &ProblemReport,
) -> Option<ShelleyAddress> {
    let Some(key) = key else {
        return None;
    };

    if key == 0 {
        report.invalid_value(
            "payment key",
            "0",
            "Payment reference key must not be 0",
            context,
        );
        return None;
    }

    let index = match decremented_index(key.abs()) {
        Ok(value) => value,
        Err(e) => {
            report.other(
                &format!("Invalid index ({key:?}) of the payment key: {e:?}"),
                context,
            );
            return None;
        },
    };

    // Negative indicates reference to transaction output.
    if key < 0 {
        convert_transaction_output(index, transaction, context, report)
    } else {
        // Positive indicates reference to tx input.
        let inputs = &transaction.transaction_body.inputs;
        // Check whether the index exists in transaction inputs.
        if inputs.get(index).is_none() {
            report.other(
                &format!(
                    "Role payment key reference index ({index}) is not found in transaction inputs"
                ),
                context,
            );
        }

        report.other(
            &format!("Payment key reference ({key:?}) to transaction input is unsupported"),
            context,
        );
        None
    }
}

/// Converts payment key transaction output reference to `ShelleyAddress`.
fn convert_transaction_output(
    index: usize, transaction: &conway::MintedTx, context: &str, report: &ProblemReport,
) -> Option<ShelleyAddress> {
    let outputs = &transaction.transaction_body.outputs;
    let transaction = MultiEraTx::Conway(Box::new(Cow::Borrowed(transaction)));
    let witness = match TxWitness::new(&[transaction]) {
        Ok(witnesses) => witnesses,
        Err(e) => {
            report.other(&format!("Failed to create TxWitness: {e:?}"), context);
            return None;
        },
    };

    let address = match outputs.get(index) {
        Some(conway::PseudoTransactionOutput::PostAlonzo(o)) => &o.address,
        Some(conway::PseudoTransactionOutput::Legacy(_)) => {
            report.other(
                &format!("Unsupported legacy transaction output type in payment key reference (index = {index})"),
                context,
            );
            return None;
        },
        None => {
            report.other(
                &format!(
                    "Role payment key reference index ({index}) is not found in transaction outputs"
                ),
                context,
            );
            return None;
        },
    };
    validate_payment_output(address, &witness, context, report);
    match Address::from_bytes(address) {
        Ok(Address::Shelley(a)) => Some(a),
        Ok(a) => {
            report.other(
                &format!(
                    "Unsupported address type ({a:?}) in payment key reference (index = {index})"
                ),
                context,
            );
            None
        },
        Err(e) => {
            report.other(
                &format!("Invalid address in payment key reference (index = {index}): {e:?}"),
                context,
            );
            None
        },
    }
}

/// Helper function for validating payment output key.
fn validate_payment_output(
    output_address: &[u8], witness: &TxWitness, context: &str, report: &ProblemReport,
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
                "Unable to find payment output key ({key:?}) in the transaction witness set: {e:?}"
            ),
            context,
        );
    }
}
