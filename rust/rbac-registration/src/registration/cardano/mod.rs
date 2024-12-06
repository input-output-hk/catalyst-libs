//! Chain of Cardano registration data

pub mod payment_history;
pub mod point_tx_idx;
pub mod role_data;

use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use c509_certificate::c509::C509;
use ed25519_dalek::VerifyingKey;
use pallas::{
    crypto::hash::Hash,
    ledger::{
        addresses::{Address, ShelleyAddress, ShelleyPaymentPart},
        traverse::MultiEraTx,
    },
    network::miniprotocols::Point,
};
use payment_history::PaymentHistory;
use point_tx_idx::PointTxIdx;
use role_data::RoleData;
use tracing::error;
use uuid::Uuid;

use crate::{
    cardano::cip509::{
        self,
        rbac::{
            certs::{C509Cert, X509DerCert},
            pub_key::SimplePublicKeyType,
        },
        types::cert_key_hash::CertKeyHash,
        Cip509, Cip509Validation,
    },
    utils::general::decremented_index,
};

/// Registration chains.
pub struct RegistrationChain {
    /// Inner part of the registration chain.
    inner: Arc<RegistrationChainInner>,
}

impl RegistrationChain {
    /// Create a new instance of registration chain.
    /// The first new value should be the chain root.
    ///
    /// # Arguments
    /// - `cip509` - The CIP509.
    /// - `tracking_payment_keys` - The list of payment keys to track.
    /// - `point` - The point (slot) of the transaction.
    /// - `tx_idx` - The transaction index.
    /// - `txn` - The transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    pub fn new(
        point: Point, tracking_payment_keys: &[ShelleyAddress], tx_idx: usize, txn: &MultiEraTx,
        cip509: Cip509,
    ) -> anyhow::Result<Self> {
        let inner = RegistrationChainInner::new(cip509, tracking_payment_keys, point, tx_idx, txn)?;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Update the registration chain.
    ///
    /// # Arguments
    /// - `point` - The point (slot) of the transaction.
    /// - `tx_idx` - The transaction index.
    /// - `txn` - The transaction.
    /// - `cip509` - The CIP509.
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    pub fn update(
        &self, point: Point, tx_idx: usize, txn: &MultiEraTx, cip509: Cip509,
    ) -> anyhow::Result<Self> {
        let new_inner = self.inner.update(point, tx_idx, txn, cip509)?;

        Ok(Self {
            inner: Arc::new(new_inner),
        })
    }

    /// Get the current transaction ID hash.
    #[must_use]
    pub fn current_tx_id_hash(&self) -> Hash<32> {
        self.inner.current_tx_id_hash
    }

    /// Get a list of purpose for this registration chain.
    #[must_use]
    pub fn purpose(&self) -> &[Uuid] {
        &self.inner.purpose
    }

    /// Get the map of index in array to point, transaction index, and x509 certificate.
    #[must_use]
    pub fn x509_certs(&self) -> &HashMap<usize, (PointTxIdx, Vec<u8>)> {
        &self.inner.x509_certs
    }

    /// Get the map of index in array to point, transaction index, and c509 certificate.
    #[must_use]
    pub fn c509_certs(&self) -> &HashMap<usize, (PointTxIdx, C509)> {
        &self.inner.c509_certs
    }

    /// Get the map of index in array to point, transaction index, and public key.
    #[must_use]
    pub fn simple_keys(&self) -> &HashMap<usize, (PointTxIdx, VerifyingKey)> {
        &self.inner.simple_keys
    }

    /// Get a list of revocations.
    #[must_use]
    pub fn revocations(&self) -> &[(PointTxIdx, CertKeyHash)] {
        &self.inner.revocations
    }

    /// Get the map of role number to point, transaction index, and role data.
    #[must_use]
    pub fn role_data(&self) -> &HashMap<u8, (PointTxIdx, RoleData)> {
        &self.inner.role_data
    }

    /// Get the map of tracked payment keys to its history.
    #[must_use]
    pub fn tracking_payment_history(&self) -> &HashMap<ShelleyAddress, Vec<PaymentHistory>> {
        &self.inner.tracking_payment_history
    }
}

/// Inner structure of registration chain.
#[derive(Clone)]
struct RegistrationChainInner {
    /// The current transaction ID hash (32 bytes)
    current_tx_id_hash: Hash<32>,
    /// List of purpose for this registration chain
    purpose: Vec<Uuid>,

    // RBAC
    /// Map of index in array to point, transaction index, and x509 certificate.
    x509_certs: HashMap<usize, (PointTxIdx, Vec<u8>)>,
    /// Map of index in array to point, transaction index, and c509 certificate.
    c509_certs: HashMap<usize, (PointTxIdx, C509)>,
    /// Map of index in array to point, transaction index, and public key.
    simple_keys: HashMap<usize, (PointTxIdx, VerifyingKey)>,
    /// List of point, transaction index, and certificate key hash.
    revocations: Vec<(PointTxIdx, CertKeyHash)>,

    // Role
    /// Map of role number to point, transaction index, and role data.
    role_data: HashMap<u8, (PointTxIdx, RoleData)>,
    /// Map of tracked payment key to its history.
    tracking_payment_history: HashMap<ShelleyAddress, Vec<PaymentHistory>>,
}

impl RegistrationChainInner {
    /// Create a new instance of registration chain.
    /// The first new value should be the chain root.
    ///
    /// # Arguments
    /// - `cip509` - The CIP509.
    /// - `tracking_payment_keys` - The list of payment keys to track.
    /// - `point` - The point (slot) of the transaction.
    /// - `tx_idx` - The transaction index.
    /// - `txn` - The transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    fn new(
        cip509: Cip509, tracking_payment_keys: &[ShelleyAddress], point: Point, tx_idx: usize,
        txn: &MultiEraTx,
    ) -> anyhow::Result<Self> {
        // Should be chain root, return immediately if not
        if cip509.prv_tx_id.is_some() {
            bail!("Invalid chain root, previous transaction ID should be None.");
        }

        let mut validation_report = Vec::new();
        let validation_data = cip509.validate(txn, &mut validation_report);

        // Do the CIP509 validation, ensuring the basic validation pass.
        if !is_valid_cip509(&validation_data) {
            // Log out the error if any
            error!("CIP509 validation failed: {:?}", validation_report);
            bail!("CIP509 validation failed, {:?}", validation_report);
        }

        // Add purpose to the list
        let purpose = vec![cip509.purpose];

        let registration = cip509.x509_chunks.0;
        let point_tx_idx = PointTxIdx::new(point, tx_idx);

        let x509_cert_map = chain_root_x509_certs(registration.x509_certs, &point_tx_idx);
        let c509_cert_map = chain_root_c509_certs(registration.c509_certs, &point_tx_idx);
        let public_key_map = chain_root_public_keys(registration.pub_keys, &point_tx_idx);
        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        let role_data_map = chain_root_role_data(registration.role_set, txn, &point_tx_idx)?;

        let mut tracking_payment_history = HashMap::new();
        // Create a payment history for each tracking payment key
        for tracking_key in tracking_payment_keys {
            tracking_payment_history.insert(tracking_key.clone(), Vec::new());
        }
        // Keep record of payment history, the payment key that we want to track
        update_tracking_payment_history(&mut tracking_payment_history, txn, &point_tx_idx)?;

        Ok(Self {
            purpose,
            current_tx_id_hash: txn.hash(),
            x509_certs: x509_cert_map,
            c509_certs: c509_cert_map,
            simple_keys: public_key_map,
            revocations,
            role_data: role_data_map,
            tracking_payment_history,
        })
    }

    /// Update the registration chain.
    ///
    /// # Arguments
    /// - `point` - The point (slot) of the transaction.
    /// - `tx_idx` - The transaction index.
    /// - `txn` - The transaction.
    /// - `cip509` - The CIP509.
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    fn update(
        &self, point: Point, tx_idx: usize, txn: &MultiEraTx, cip509: Cip509,
    ) -> anyhow::Result<Self> {
        let mut new_inner = self.clone();

        let mut validation_report = Vec::new();
        let validation_data = cip509.validate(txn, &mut validation_report);

        // Do the CIP509 validation, ensuring the basic validation pass.
        if !is_valid_cip509(&validation_data) {
            error!("CIP509 validation failed: {:?}", validation_report);
            bail!("CIP509 validation failed, {:?}", validation_report);
        }

        // Check and update the current transaction ID hash
        if let Some(prv_tx_id) = cip509.prv_tx_id {
            // Previous transaction ID in the CIP509 should equal to the current transaction ID
            // or else it is not a part of the chain
            if prv_tx_id == self.current_tx_id_hash {
                new_inner.current_tx_id_hash = prv_tx_id;
            } else {
                bail!("Invalid previous transaction ID, not a part of this registration chain");
            }
        }

        // Add purpose to the chain, if not already exist
        let purpose = cip509.purpose;
        if !self.purpose.contains(&purpose) {
            new_inner.purpose.push(purpose);
        }

        let registration = cip509.x509_chunks.0;
        let point_tx_idx = PointTxIdx::new(point, tx_idx);

        update_x509_certs(&mut new_inner, registration.x509_certs, &point_tx_idx);
        update_c509_certs(&mut new_inner, registration.c509_certs, &point_tx_idx)?;
        update_public_keys(&mut new_inner, registration.pub_keys, &point_tx_idx);

        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        // Revocation list should be appended
        new_inner.revocations.extend(revocations);

        update_role_data(&mut new_inner, registration.role_set, txn, &point_tx_idx)?;

        update_tracking_payment_history(
            &mut new_inner.tracking_payment_history,
            txn,
            &point_tx_idx,
        )?;

        Ok(new_inner)
    }
}

/// Check if the CIP509 is valid.
fn is_valid_cip509(validation_data: &Cip509Validation) -> bool {
    validation_data.valid_aux
        && validation_data.valid_txn_inputs_hash
        && validation_data.valid_public_key
        && validation_data.valid_payment_key
        && validation_data.signing_key
}

/// Process x509 certificate for chain root.
fn chain_root_x509_certs(
    x509_certs: Option<Vec<X509DerCert>>, point_tx_idx: &PointTxIdx,
) -> HashMap<usize, (PointTxIdx, Vec<u8>)> {
    let mut map = HashMap::new();
    if let Some(cert_list) = x509_certs {
        for (idx, cert) in cert_list.iter().enumerate() {
            // Chain root, expect only the certificate not undefined or delete
            if let cip509::rbac::certs::X509DerCert::X509Cert(cert) = cert {
                map.insert(idx, (point_tx_idx.clone(), cert.clone()));
            }
        }
    }
    map
}

/// Update x509 certificates in the registration chain.
fn update_x509_certs(
    new_inner: &mut RegistrationChainInner, x509_certs: Option<Vec<X509DerCert>>,
    point_tx_idx: &PointTxIdx,
) {
    if let Some(cert_list) = x509_certs {
        for (idx, cert) in cert_list.iter().enumerate() {
            match cert {
                // Unchanged to that index, so continue
                cip509::rbac::certs::X509DerCert::Undefined => continue,
                // Delete the certificate
                cip509::rbac::certs::X509DerCert::Deleted => {
                    new_inner.x509_certs.remove(&idx);
                },
                // Add the new certificate
                cip509::rbac::certs::X509DerCert::X509Cert(cert) => {
                    new_inner
                        .x509_certs
                        .insert(idx, (point_tx_idx.clone(), cert.clone()));
                },
            }
        }
    }
}

/// Process c509 certificates for chain root.
fn chain_root_c509_certs(
    c509_certs: Option<Vec<C509Cert>>, point_tx_idx: &PointTxIdx,
) -> HashMap<usize, (PointTxIdx, C509)> {
    let mut map = HashMap::new();
    if let Some(cert_list) = c509_certs {
        for (idx, cert) in cert_list.iter().enumerate() {
            if let cip509::rbac::certs::C509Cert::C509Certificate(cert) = cert {
                // Chain root, expect only the certificate not undefined or delete
                map.insert(idx, (point_tx_idx.clone(), *cert.clone()));
            }
        }
    }
    map
}

/// Update c509 certificates in the registration chain.
fn update_c509_certs(
    new_inner: &mut RegistrationChainInner, c509_certs: Option<Vec<C509Cert>>,
    point_tx_idx: &PointTxIdx,
) -> anyhow::Result<()> {
    if let Some(cert_list) = c509_certs {
        for (idx, cert) in cert_list.iter().enumerate() {
            match cert {
                // Unchanged to that index, so continue
                cip509::rbac::certs::C509Cert::Undefined => continue,
                // Delete the certificate
                cip509::rbac::certs::C509Cert::Deleted => {
                    new_inner.c509_certs.remove(&idx);
                },
                // Certificate reference
                cip509::rbac::certs::C509Cert::C509CertInMetadatumReference(_) => {
                    bail!("Unsupported c509 certificate in metadatum reference")
                },
                // Add the new certificate
                cip509::rbac::certs::C509Cert::C509Certificate(c509) => {
                    new_inner
                        .c509_certs
                        .insert(idx, (point_tx_idx.clone(), *c509.clone()));
                },
            }
        }
    }
    Ok(())
}

/// Process public keys for chain root.
fn chain_root_public_keys(
    pub_keys: Option<Vec<SimplePublicKeyType>>, point_tx_idx: &PointTxIdx,
) -> HashMap<usize, (PointTxIdx, VerifyingKey)> {
    let mut map = HashMap::new();
    if let Some(key_list) = pub_keys {
        for (idx, key) in key_list.iter().enumerate() {
            // Chain root, expect only the public key not undefined or delete
            if let cip509::rbac::pub_key::SimplePublicKeyType::Ed25519(key) = key {
                map.insert(idx, (point_tx_idx.clone(), *key));
            }
        }
    }
    map
}

/// Update public keys in the registration chain.
fn update_public_keys(
    new_inner: &mut RegistrationChainInner, pub_keys: Option<Vec<SimplePublicKeyType>>,
    point_tx_idx: &PointTxIdx,
) {
    if let Some(key_list) = pub_keys {
        for (idx, cert) in key_list.iter().enumerate() {
            match cert {
                // Unchanged to that index, so continue
                cip509::rbac::pub_key::SimplePublicKeyType::Undefined => continue,
                // Delete the public key
                cip509::rbac::pub_key::SimplePublicKeyType::Deleted => {
                    new_inner.simple_keys.remove(&idx);
                },
                // Add the new public key
                cip509::rbac::pub_key::SimplePublicKeyType::Ed25519(key) => {
                    new_inner
                        .simple_keys
                        .insert(idx, (point_tx_idx.clone(), *key));
                },
            }
        }
    }
}

/// Process the revocation list.
fn revocations_list(
    revocation_list: Option<Vec<CertKeyHash>>, point_tx_idx: &PointTxIdx,
) -> Vec<(PointTxIdx, CertKeyHash)> {
    let mut revocations = Vec::new();
    if let Some(revocations_data) = revocation_list {
        for item in revocations_data {
            revocations.push((point_tx_idx.clone(), item.clone()));
        }
    }
    revocations
}

/// Process the role data for chain root.
fn chain_root_role_data(
    role_set: Option<Vec<cip509::rbac::role_data::RoleData>>, txn: &MultiEraTx,
    point_tx_idx: &PointTxIdx,
) -> anyhow::Result<HashMap<u8, (PointTxIdx, RoleData)>> {
    let mut role_data_map = HashMap::new();
    if let Some(role_set_data) = role_set {
        for role_data in role_set_data {
            let signing_key = role_data.role_signing_key.clone();
            let encryption_key = role_data.role_encryption_key.clone();

            // Get the payment key
            let payment_key = get_payment_addr_from_tx(txn, role_data.payment_key)?;

            // Map of role number to point and role data
            role_data_map.insert(
                role_data.role_number,
                (
                    point_tx_idx.clone(),
                    RoleData::new(
                        signing_key,
                        encryption_key,
                        payment_key,
                        role_data.role_extended_data_keys.clone(),
                    ),
                ),
            );
        }
    }
    Ok(role_data_map)
}

/// Update the role data in the registration chain.
fn update_role_data(
    inner: &mut RegistrationChainInner, role_set: Option<Vec<cip509::rbac::role_data::RoleData>>,
    txn: &MultiEraTx, point_tx_idx: &PointTxIdx,
) -> anyhow::Result<()> {
    if let Some(role_set_data) = role_set {
        for role_data in role_set_data {
            // If there is new role singing key, use it, else use the old one
            let signing_key = match role_data.role_signing_key {
                Some(key) => Some(key),
                None => {
                    match inner.role_data.get(&role_data.role_number) {
                        Some((_, role_data)) => role_data.signing_key_ref().clone(),
                        None => None,
                    }
                },
            };

            // If there is new role encryption key, use it, else use the old one
            let encryption_key = match role_data.role_encryption_key {
                Some(key) => Some(key),
                None => {
                    match inner.role_data.get(&role_data.role_number) {
                        Some((_, role_data)) => role_data.encryption_ref().clone(),
                        None => None,
                    }
                },
            };
            let payment_key = get_payment_addr_from_tx(txn, role_data.payment_key)?;

            // Map of role number to point and role data
            // Note that new role data will overwrite the old one
            inner.role_data.insert(
                role_data.role_number,
                (
                    point_tx_idx.clone(),
                    RoleData::new(
                        signing_key,
                        encryption_key,
                        payment_key,
                        role_data.role_extended_data_keys.clone(),
                    ),
                ),
            );
        }
    }
    Ok(())
}

/// Helper function for retrieving the Shelley address from the transaction.
fn get_payment_addr_from_tx(
    txn: &MultiEraTx, payment_key_ref: Option<i16>,
) -> anyhow::Result<Option<ShelleyPaymentPart>> {
    // The index should exist since it pass the basic validation
    if let Some(key_ref) = payment_key_ref {
        if let MultiEraTx::Conway(tx) = txn {
            // Transaction output
            if key_ref < 0 {
                let index = decremented_index(key_ref.abs())?;
                if let Some(output) = tx.transaction_body.outputs.get(index) {
                    // Conway era -> Post alonzo tx output
                    match output {
                        pallas::ledger::primitives::conway::PseudoTransactionOutput::PostAlonzo(
                            o,
                        ) => {
                            let address =
                                Address::from_bytes(&o.address).map_err(|e| anyhow::anyhow!(e))?;

                            if let Address::Shelley(addr) = address {
                                return Ok(Some(addr.payment().clone()));
                            }
                            bail!("Unsupported address type in payment key reference");
                        },
                        // Not support legacy form of transaction output
                        pallas::ledger::primitives::conway::PseudoTransactionOutput::Legacy(_) => {
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
    }
    Ok(None)
}

/// Update the payment history given the tracking payment keys.
fn update_tracking_payment_history(
    tracking_payment_history: &mut HashMap<ShelleyAddress, Vec<PaymentHistory>>, txn: &MultiEraTx,
    point_tx_idx: &PointTxIdx,
) -> anyhow::Result<()> {
    if let MultiEraTx::Conway(tx) = txn {
        // Conway era -> Post alonzo tx output
        for (index, output) in tx.transaction_body.outputs.iter().enumerate() {
            match output {
                pallas::ledger::primitives::conway::PseudoTransactionOutput::PostAlonzo(o) => {
                    let address =
                        Address::from_bytes(&o.address).map_err(|e| anyhow::anyhow!(e))?;
                    let shelley_payment = if let Address::Shelley(addr) = address {
                        addr.clone()
                    } else {
                        bail!("Unsupported address type in update payment history");
                    };
                    // If the payment key from the output exist in the payment history, add the
                    // history
                    if let Some(vec) = tracking_payment_history.get_mut(&shelley_payment) {
                        let output_index: u16 = index.try_into().map_err(|_| {
                            anyhow::anyhow!("Cannot convert usize to u16 in update payment history")
                        })?;

                        vec.push(PaymentHistory::new(
                            point_tx_idx.clone(),
                            txn.hash(),
                            output_index,
                            o.value.clone(),
                        ));
                    }
                },
                pallas::ledger::primitives::conway::PseudoTransactionOutput::Legacy(_) => {
                    bail!("Unsupported transaction output type in update payment history");
                },
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use minicbor::{Decode, Decoder};
    use pallas::{ledger::traverse::MultiEraTx, network::miniprotocols::Point};

    use super::RegistrationChain;
    use crate::cardano::{cip509::Cip509, transaction::raw_aux_data::RawAuxData};

    fn cip_509_aux_data(tx: &MultiEraTx<'_>) -> Vec<u8> {
        let raw_auxiliary_data = tx
            .as_conway()
            .unwrap()
            .clone()
            .auxiliary_data
            .map(|aux| aux.raw_cbor());

        let raw_cbor_data = match raw_auxiliary_data {
            pallas::codec::utils::Nullable::Some(data) => Ok(data),
            _ => Err("Auxiliary data not found"),
        };

        let auxiliary_data = RawAuxData::new(raw_cbor_data.expect("Failed to get raw cbor data"));
        auxiliary_data
            .get_metadata(509)
            .expect("Failed to get metadata")
            .to_vec()
    }

    fn conway_1() -> Vec<u8> {
        hex::decode(include_str!("../../test_data/cardano/conway_1.block"))
            .expect("Failed to decode hex block.")
    }

    fn conway_4() -> Vec<u8> {
        hex::decode(include_str!("../../test_data/cardano/conway_4.block"))
            .expect("Failed to decode hex block.")
    }

    #[test]
    fn test_new_and_update_registration() {
        let conway_block_data_1 = conway_1();
        let point_1 = Point::new(
            77_429_134,
            hex::decode("62483f96613b4c48acd28de482eb735522ac180df61766bdb476a7bf83e7bb98")
                .unwrap(),
        );
        let multi_era_block_1 =
            pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data_1)
                .expect("Failed to decode MultiEraBlock");

        let transactions_1 = multi_era_block_1.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx_1 = transactions_1
            .get(3)
            .expect("Failed to get transaction index");

        let aux_data_1 = cip_509_aux_data(tx_1);
        let mut decoder = Decoder::new(aux_data_1.as_slice());
        let cip509_1 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");
        let tracking_payment_keys = vec![];

        let registration_chain =
            RegistrationChain::new(point_1.clone(), &tracking_payment_keys, 3, tx_1, cip509_1);
        // Able to add chain root to the registration chain
        assert!(registration_chain.is_ok());

        let conway_block_data_4 = conway_4();
        let point_4 = Point::new(
            77_436_369,
            hex::decode("b174fc697126f05046b847d47e60d66cbedaf25240027f9c07f27150889aac24")
                .unwrap(),
        );

        let multi_era_block_4 =
            pallas::ledger::traverse::MultiEraBlock::decode(&conway_block_data_4)
                .expect("Failed to decode MultiEraBlock");

        let transactions_4 = multi_era_block_4.txs();
        // Second transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions_4
            .get(1)
            .expect("Failed to get transaction index");

        let aux_data_4 = cip_509_aux_data(tx);
        let mut decoder = Decoder::new(aux_data_4.as_slice());
        let cip509 = Cip509::decode(&mut decoder, &mut ()).expect("Failed to decode Cip509");

        // Update the registration chain
        assert!(registration_chain
            .unwrap()
            .update(point_4.clone(), 1, tx, cip509)
            .is_ok());
    }
}
