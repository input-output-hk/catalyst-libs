//! Chain of Cardano registration data

use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use c509_certificate::c509::C509;
use pallas::{
    codec::utils::Bytes,
    crypto::hash::Hash,
    ledger::{primitives::conway::Value, traverse::MultiEraTx},
    network::miniprotocols::Point,
};
use tracing::error;

use crate::{
    cardano::cip509::{
        self,
        rbac::{
            certs::{C509Cert, X509DerCert},
            pub_key::{Ed25519PublicKey, SimplePublicKeyType},
            role_data::KeyLocalRef,
            CertKeyHash,
        },
        Cip509, UuidV4,
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
        &self, point: Point, tracking_payment_keys: Vec<Ed25519PublicKey>, tx_idx: usize,
        txn: &MultiEraTx, cip509: Cip509,
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

    /// Get the registration chain inner.
    #[must_use]
    pub fn registration_chain(&self) -> &RegistrationChainInner {
        self.inner.as_ref()
    }
}

/// Inner structure of registration chain.
#[derive(Clone)]
pub struct RegistrationChainInner {
    /// The current transaction ID hash (32 bytes)
    current_tx_id_hash: Hash<32>,
    /// List of purpose for this registration chain
    purpose: Vec<UuidV4>,

    // RBAC
    /// Map of index in array to point, transaction index, and x509 certificate.
    x509_certs: HashMap<usize, (PointTxIdx, Vec<u8>)>,
    /// Map of index in array to point, transaction index, and c509 certificate.
    c509_certs: HashMap<usize, (PointTxIdx, C509)>,
    /// Map of index in array to point, transaction index, and public key.
    simple_keys: HashMap<usize, (PointTxIdx, Ed25519PublicKey)>,
    /// List of point, transaction index, and certificate key hash.
    revocations: Vec<(PointTxIdx, CertKeyHash)>,

    // Role
    /// Map of role number to point, transaction index, and role data.
    role_data: HashMap<u8, (PointTxIdx, RoleData)>,
    /// List of payment keys to track.
    tracking_payment_keys: Arc<Vec<Ed25519PublicKey>>,
    /// Map of payment key to its history.
    payment_history: HashMap<Ed25519PublicKey, Vec<PaymentHistory>>,
}

/// Point (slot) and transaction index.
#[derive(Clone)]
pub struct PointTxIdx((Point, usize));

impl PointTxIdx {
    /// Create an instance of point and transaction index.
    pub(crate) fn new(point: Point, tx_idx: usize) -> Self {
        PointTxIdx((point, tx_idx))
    }

    /// Get the point.
    #[must_use]
    pub fn point(&self) -> &Point {
        &self.0 .0
    }

    /// Get the transaction index.
    #[must_use]
    pub fn tx_idx(&self) -> usize {
        self.0 .1
    }
}

/// Payment history of the public key in tracking payment keys.
#[derive(Clone)]
pub struct PaymentHistory {
    /// The point and transaction index.
    point_tx_idx: PointTxIdx,
    /// Transaction hash that this payment come from.
    tx_hash: Hash<32>,
    /// The transaction output index that this payment come from.
    output_index: u16,
    /// The value of the payment.
    value: Value,
}

impl PaymentHistory {
    /// Get the point and transaction index.
    #[must_use]
    pub fn point_tx_idx(&self) -> &PointTxIdx {
        &self.point_tx_idx
    }

    /// Get the transaction hash.
    #[must_use]
    pub fn tx_hash(&self) -> Hash<32> {
        self.tx_hash
    }

    /// Get the transaction output index.
    #[must_use]
    pub fn output_index(&self) -> u16 {
        self.output_index
    }

    /// Get the value of the payment.
    #[must_use]
    pub fn value(&self) -> &Value {
        &self.value
    }
}

/// Role data
#[derive(Clone)]
pub struct RoleData {
    /// List of reference of signing keys to the data within registration.
    signing_key_ref: Vec<KeyLocalRef>,
    /// List of reference of encryption keys to the data within registration.
    encryption_ref: Vec<KeyLocalRef>,
    /// A payment key where reward will be distributed to.
    payment_key: Ed25519PublicKey,
    /// Map of role extended data (10-99) to its data
    role_extended_data: HashMap<u8, Vec<u8>>,
}

impl RoleData {
    /// Get the reference of signing keys.
    #[must_use]
    pub fn signing_key_ref(&self) -> &[KeyLocalRef] {
        &self.signing_key_ref
    }

    /// Get the reference of encryption keys.
    #[must_use]
    pub fn encryption_ref(&self) -> &[KeyLocalRef] {
        &self.encryption_ref
    }

    /// Get the payment key.
    #[must_use]
    pub fn payment_key(&self) -> &Ed25519PublicKey {
        &self.payment_key
    }

    /// Get the role extended data.
    #[must_use]
    pub fn role_extended_data(&self) -> &HashMap<u8, Vec<u8>> {
        &self.role_extended_data
    }
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
        cip509: Cip509, tracking_payment_keys: Vec<Ed25519PublicKey>, point: Point, tx_idx: usize,
        txn: &MultiEraTx,
    ) -> anyhow::Result<Self> {
        // Should be chain root, return immediately if not
        if cip509.prv_tx_id.is_some() {
            bail!("Invalid chain root, previous transaction ID should be None.");
        }

        let mut validation_report = Vec::new();
        // Do the CIP509 validation, ensuring the basic validation pass.
        if !cip509.validate(txn, tx_idx, &mut validation_report) {
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

        let mut payment_history = HashMap::new();
        for tracking_key in &tracking_payment_keys {
            // Keep record of payment history, the payment key that we want to track
            let histories = update_payment_history(tracking_key, txn, &point_tx_idx)?;
            payment_history.insert(tracking_key.clone(), histories);
        }

        Ok(Self {
            purpose,
            current_tx_id_hash: txn.hash(),
            x509_certs: x509_cert_map,
            c509_certs: c509_cert_map,
            simple_keys: public_key_map,
            revocations,
            role_data: role_data_map,
            tracking_payment_keys: Arc::new(tracking_payment_keys),
            payment_history,
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
        let mut new_inner = self.clone();

        let mut validation_report = Vec::new();
        // Do the CIP509 validation, ensuring the basic validation pass.
        if !cip509.validate(txn, tx_idx, &mut validation_report) {
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

        for tracking_key in self.tracking_payment_keys.iter() {
            let histories = update_payment_history(tracking_key, txn, &point_tx_idx)?;
            // If tracking payment key doesn't exist, insert an empty vector,
            // then add the histories to the history vector
            new_inner
                .payment_history
                .entry(tracking_key.clone())
                .or_default()
                .extend(histories);
        }

        Ok(new_inner)
    }

    /// Get the current transaction ID hash.
    #[must_use]
    pub fn current_tx_id_hash(&self) -> Hash<32> {
        self.current_tx_id_hash
    }

    /// Get a list of purpose for this registration chain.
    #[must_use]
    pub fn purpose(&self) -> &[UuidV4] {
        &self.purpose
    }

    /// Get the map of index in array to point, transaction index, and x509 certificate.
    #[must_use]
    pub fn x509_certs(&self) -> &HashMap<usize, (PointTxIdx, Vec<u8>)> {
        &self.x509_certs
    }

    /// Get the map of index in array to point, transaction index, and c509 certificate.
    #[must_use]
    pub fn c509_certs(&self) -> &HashMap<usize, (PointTxIdx, C509)> {
        &self.c509_certs
    }

    /// Get the map of index in array to point, transaction index, and public key.
    #[must_use]
    pub fn simple_keys(&self) -> &HashMap<usize, (PointTxIdx, Ed25519PublicKey)> {
        &self.simple_keys
    }

    /// Get a list of revocations.
    #[must_use]
    pub fn revocations(&self) -> &[(PointTxIdx, CertKeyHash)] {
        &self.revocations
    }

    /// Get the map of role number to point, transaction index, and role data.
    #[must_use]
    pub fn role_data(&self) -> &HashMap<u8, (PointTxIdx, RoleData)> {
        &self.role_data
    }

    /// Get the list of payment keys to track.
    #[must_use]
    pub fn tracking_payment_keys(&self) -> &Vec<Ed25519PublicKey> {
        &self.tracking_payment_keys
    }

    /// Get the map of payment key to its history.
    #[must_use]
    pub fn payment_history(&self) -> &HashMap<Ed25519PublicKey, Vec<PaymentHistory>> {
        &self.payment_history
    }
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
) -> HashMap<usize, (PointTxIdx, Ed25519PublicKey)> {
    let mut map = HashMap::new();
    if let Some(key_list) = pub_keys {
        for (idx, key) in key_list.iter().enumerate() {
            // Chain root, expect only the public key not undefined or delete
            if let cip509::rbac::pub_key::SimplePublicKeyType::Ed25519(key) = key {
                map.insert(idx, (point_tx_idx.clone(), key.clone()));
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
                        .insert(idx, (point_tx_idx.clone(), key.clone()));
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
            let signing_key = role_data.role_signing_key.clone().unwrap_or_default();
            let encryption_key = role_data.role_encryption_key.clone().unwrap_or_default();

            // Get the payment key
            let payment_key = get_payment_key_from_tx(txn, role_data.payment_key)?;

            // Map of role number to point and role data
            role_data_map.insert(
                role_data.role_number,
                (point_tx_idx.clone(), RoleData {
                    signing_key_ref: signing_key,
                    encryption_ref: encryption_key,
                    payment_key,
                    role_extended_data: role_data.role_extended_data_keys.clone(),
                }),
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
            let signing_key = role_data.role_signing_key.unwrap_or_else(|| {
                match inner.role_data.get(&role_data.role_number) {
                    Some((_, role_data)) => role_data.signing_key_ref.clone(),
                    None => Vec::new(),
                }
            });

            // If there is new role encryption key, use it, else use the old one
            let encryption_key = role_data.role_encryption_key.unwrap_or_else(|| {
                match inner.role_data.get(&role_data.role_number) {
                    Some((_, role_data)) => role_data.encryption_ref.clone(),
                    None => Vec::new(),
                }
            });
            let payment_key = get_payment_key_from_tx(txn, role_data.payment_key)?;

            // Map of role number to point and role data
            // Note that new role data will overwrite the old one
            inner.role_data.insert(
                role_data.role_number,
                (point_tx_idx.clone(), RoleData {
                    signing_key_ref: signing_key,
                    encryption_ref: encryption_key,
                    payment_key,
                    role_extended_data: role_data.role_extended_data_keys.clone(),
                }),
            );
        }
    }
    Ok(())
}

/// Helper function for retrieving the payment key from the transaction.
fn get_payment_key_from_tx(
    txn: &MultiEraTx, payment_key_ref: Option<i16>,
) -> anyhow::Result<Ed25519PublicKey> {
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
                            let payment_key: Ed25519PublicKey =
                                o.address.clone().try_into().map_err(|_| {
                                    anyhow::anyhow!("Failed to convert Vec<u8> to Ed25519PublicKey in payment key reference")
                                })?;
                            return Ok(payment_key);
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
    Ok(Ed25519PublicKey::default())
}

/// Update the payment history given the tracking payment keys.
fn update_payment_history(
    tracking_key: &Ed25519PublicKey, txn: &MultiEraTx, point_tx_idx: &PointTxIdx,
) -> anyhow::Result<Vec<PaymentHistory>> {
    let mut payment_history = Vec::new();
    if let MultiEraTx::Conway(tx) = txn {
        // Conway era -> Post alonzo tx output
        for (index, output) in tx.transaction_body.outputs.iter().enumerate() {
            match output {
                pallas::ledger::primitives::conway::PseudoTransactionOutput::PostAlonzo(o) => {
                    let address_bytes: Bytes = tracking_key.clone().into();
                    if address_bytes == o.address {
                        let output_index: u16 = index.try_into().map_err(|_| {
                            anyhow::anyhow!("Cannot convert usize to u16 in update payment history")
                        })?;

                        payment_history.push(PaymentHistory {
                            point_tx_idx: point_tx_idx.clone(),
                            tx_hash: txn.hash(),
                            output_index,
                            value: o.value.clone(),
                        });
                    }
                },
                pallas::ledger::primitives::conway::PseudoTransactionOutput::Legacy(_) => {
                    bail!("Unsupported transaction output type in update payment history");
                },
            }
        }
    }
    Ok(payment_history)
}
