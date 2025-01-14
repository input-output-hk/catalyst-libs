//! Chain of Cardano registration data

use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use c509_certificate::c509::C509;
use cardano_blockchain_types::hashes::Blake2b256Hash;
use ed25519_dalek::VerifyingKey;
use pallas::{
    ledger::{
        addresses::{Address, ShelleyAddress},
        traverse::MultiEraTx,
    },
    network::miniprotocols::Point,
};
use tracing::{error, warn};
use uuid::Uuid;
use x509_cert::certificate::Certificate as X509Certificate;

use crate::cardano::cip509::{
    C509Cert, CertKeyHash, Cip0134UriSet, Cip509, PaymentHistory, PointTxIdx, RoleData, RoleNumber,
    SimplePublicKeyType, X509DerCert,
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
    pub fn current_tx_id_hash(&self) -> Blake2b256Hash {
        self.inner.current_tx_id_hash
    }

    /// Get a list of purpose for this registration chain.
    #[must_use]
    pub fn purpose(&self) -> &[Uuid] {
        &self.inner.purpose
    }

    /// Get the map of index in array to point, transaction index, and x509 certificate.
    #[must_use]
    pub fn x509_certs(&self) -> &HashMap<usize, (PointTxIdx, X509Certificate)> {
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
    pub fn role_data(&self) -> &HashMap<RoleNumber, (PointTxIdx, RoleData)> {
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
    current_tx_id_hash: Blake2b256Hash,
    /// List of purpose for this registration chain
    purpose: Vec<Uuid>,

    // RBAC
    /// Map of index in array to point, transaction index, and x509 certificate.
    x509_certs: HashMap<usize, (PointTxIdx, X509Certificate)>,
    /// Map of index in array to point, transaction index, and c509 certificate.
    c509_certs: HashMap<usize, (PointTxIdx, C509)>,
    /// A set of URIs contained in both x509 and c509 certificates.
    certificate_uris: Cip0134UriSet,
    /// Map of index in array to point, transaction index, and public key.
    simple_keys: HashMap<usize, (PointTxIdx, VerifyingKey)>,
    /// List of point, transaction index, and certificate key hash.
    revocations: Vec<(PointTxIdx, CertKeyHash)>,

    // Role
    /// Map of role number to point, transaction index, and role data.
    role_data: HashMap<RoleNumber, (PointTxIdx, RoleData)>,
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
        if cip509.previous_transaction().is_some() {
            bail!("Invalid chain root, previous transaction ID should be None.");
        }

        let (purpose, registration) = match cip509.consume() {
            Ok(v) => v,
            Err(e) => {
                let error = format!("Invalid Cip509: {e:?}");
                error!(error);
                bail!(error);
            },
        };

        let purpose = vec![purpose];

        let point_tx_idx = PointTxIdx::new(point, tx_idx);

        let certificate_uris = registration.certificate_uris;
        let x509_cert_map = chain_root_x509_certs(registration.x509_certs, &point_tx_idx);
        let c509_cert_map = chain_root_c509_certs(registration.c509_certs, &point_tx_idx);
        let public_key_map = chain_root_public_keys(registration.pub_keys, &point_tx_idx);
        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        let role_data_map = chain_root_role_data(registration.role_data, &point_tx_idx);

        let mut tracking_payment_history = HashMap::new();
        // Create a payment history for each tracking payment key
        for tracking_key in tracking_payment_keys {
            tracking_payment_history.insert(tracking_key.clone(), Vec::new());
        }
        // Keep record of payment history, the payment key that we want to track
        update_tracking_payment_history(&mut tracking_payment_history, txn, &point_tx_idx)?;

        Ok(Self {
            purpose,
            current_tx_id_hash: txn.hash().into(),
            x509_certs: x509_cert_map,
            c509_certs: c509_cert_map,
            certificate_uris,
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

        let Some(prv_tx_id) = cip509.previous_transaction() else {
            bail!("Empty previous transaction ID");
        };
        // Previous transaction ID in the CIP509 should equal to the current transaction ID
        // or else it is not a part of the chain
        if prv_tx_id == self.current_tx_id_hash {
            new_inner.current_tx_id_hash = prv_tx_id;
        } else {
            bail!("Invalid previous transaction ID, not a part of this registration chain");
        }

        let (purpose, registration) = match cip509.consume() {
            Ok(v) => v,
            Err(e) => {
                let error = format!("Invalid Cip509: {e:?}");
                error!(error);
                bail!(error);
            },
        };

        // Add purpose to the chain, if not already exist
        if !self.purpose.contains(&purpose) {
            new_inner.purpose.push(purpose);
        }

        let point_tx_idx = PointTxIdx::new(point, tx_idx);
        new_inner.certificate_uris = new_inner.certificate_uris.update(&registration);
        update_x509_certs(&mut new_inner, registration.x509_certs, &point_tx_idx);
        update_c509_certs(&mut new_inner, registration.c509_certs, &point_tx_idx);
        update_public_keys(&mut new_inner, registration.pub_keys, &point_tx_idx);

        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        // Revocation list should be appended
        new_inner.revocations.extend(revocations);

        update_role_data(&mut new_inner, registration.role_data, &point_tx_idx);

        update_tracking_payment_history(
            &mut new_inner.tracking_payment_history,
            txn,
            &point_tx_idx,
        )?;

        Ok(new_inner)
    }
}

/// Process x509 certificate for chain root.
fn chain_root_x509_certs(
    x509_certs: Vec<X509DerCert>, point_tx_idx: &PointTxIdx,
) -> HashMap<usize, (PointTxIdx, X509Certificate)> {
    x509_certs
        .into_iter()
        .enumerate()
        .filter_map(|(index, cert)| {
            if let X509DerCert::X509Cert(cert) = cert {
                Some((index, (point_tx_idx.clone(), *cert)))
            } else {
                None
            }
        })
        .collect()
}

/// Update x509 certificates in the registration chain.
fn update_x509_certs(
    new_inner: &mut RegistrationChainInner, x509_certs: Vec<X509DerCert>, point_tx_idx: &PointTxIdx,
) {
    for (idx, cert) in x509_certs.into_iter().enumerate() {
        match cert {
            // Unchanged to that index, so continue
            X509DerCert::Undefined => continue,
            // Delete the certificate
            X509DerCert::Deleted => {
                new_inner.x509_certs.remove(&idx);
            },
            // Add the new certificate
            X509DerCert::X509Cert(cert) => {
                new_inner
                    .x509_certs
                    .insert(idx, (point_tx_idx.clone(), *cert));
            },
        }
    }
}

/// Process c509 certificates for chain root.
fn chain_root_c509_certs(
    c509_certs: Vec<C509Cert>, point_tx_idx: &PointTxIdx,
) -> HashMap<usize, (PointTxIdx, C509)> {
    let mut map = HashMap::new();
    for (idx, cert) in c509_certs.into_iter().enumerate() {
        if let C509Cert::C509Certificate(cert) = cert {
            // Chain root, expect only the certificate not undefined or delete
            map.insert(idx, (point_tx_idx.clone(), *cert));
        }
    }
    map
}

/// Update c509 certificates in the registration chain.
fn update_c509_certs(
    new_inner: &mut RegistrationChainInner, c509_certs: Vec<C509Cert>, point_tx_idx: &PointTxIdx,
) {
    for (idx, cert) in c509_certs.into_iter().enumerate() {
        match cert {
            // Unchanged to that index, so continue
            C509Cert::Undefined => continue,
            // Delete the certificate
            C509Cert::Deleted => {
                new_inner.c509_certs.remove(&idx);
            },
            // Certificate reference
            C509Cert::C509CertInMetadatumReference(_) => {
                warn!("Unsupported C509CertInMetadatumReference");
            },
            // Add the new certificate
            C509Cert::C509Certificate(c509) => {
                new_inner
                    .c509_certs
                    .insert(idx, (point_tx_idx.clone(), *c509));
            },
        }
    }
}

/// Process public keys for chain root.
fn chain_root_public_keys(
    pub_keys: Vec<SimplePublicKeyType>, point_tx_idx: &PointTxIdx,
) -> HashMap<usize, (PointTxIdx, VerifyingKey)> {
    let mut map = HashMap::new();
    for (idx, key) in pub_keys.into_iter().enumerate() {
        // Chain root, expect only the public key not undefined or delete
        if let SimplePublicKeyType::Ed25519(key) = key {
            map.insert(idx, (point_tx_idx.clone(), key));
        }
    }
    map
}

/// Update public keys in the registration chain.
fn update_public_keys(
    new_inner: &mut RegistrationChainInner, pub_keys: Vec<SimplePublicKeyType>,
    point_tx_idx: &PointTxIdx,
) {
    for (idx, cert) in pub_keys.into_iter().enumerate() {
        match cert {
            // Unchanged to that index, so continue
            SimplePublicKeyType::Undefined => continue,
            // Delete the public key
            SimplePublicKeyType::Deleted => {
                new_inner.simple_keys.remove(&idx);
            },
            // Add the new public key
            SimplePublicKeyType::Ed25519(key) => {
                new_inner
                    .simple_keys
                    .insert(idx, (point_tx_idx.clone(), key));
            },
        }
    }
}

/// Process the revocation list.
fn revocations_list(
    revocation_list: Vec<CertKeyHash>, point_tx_idx: &PointTxIdx,
) -> Vec<(PointTxIdx, CertKeyHash)> {
    let mut revocations = Vec::new();
    for item in revocation_list {
        revocations.push((point_tx_idx.clone(), item.clone()));
    }
    revocations
}

/// Process the role data for chain root.
fn chain_root_role_data(
    role_data: HashMap<RoleNumber, RoleData>, point_tx_idx: &PointTxIdx,
) -> HashMap<RoleNumber, (PointTxIdx, RoleData)> {
    role_data
        .into_iter()
        .map(|(number, data)| (number, (point_tx_idx.clone(), data)))
        .collect()
}

/// Update the role data in the registration chain.
fn update_role_data(
    inner: &mut RegistrationChainInner, role_set: HashMap<RoleNumber, RoleData>,
    point_tx_idx: &PointTxIdx,
) {
    for (number, mut data) in role_set {
        // If there is new role singing key, use it, else use the old one
        if data.signing_key().is_none() {
            let signing_key = inner
                .role_data
                .get(&number)
                .and_then(|(_, d)| d.signing_key())
                .cloned();
            data.set_signing_key(signing_key);
        }

        // If there is new role encryption key, use it, else use the old one
        if data.encryption_key().is_none() {
            let signing_key = inner
                .role_data
                .get(&number)
                .and_then(|(_, d)| d.encryption_key())
                .cloned();
            data.set_encryption_key(signing_key);
        }

        // Map of role number to point and role data
        // Note that new role data will overwrite the old one
        inner.role_data.insert(number, (point_tx_idx.clone(), data));
    }
}

// TODO: FIXME: Move payment history into Cip509.
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
    use super::*;

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

        let cip509_1 = Cip509::new(&multi_era_block_1, 3.into())
            .expect("Failed to decode Cip509")
            .unwrap();
        assert!(
            !cip509_1.report().is_problematic(),
            "Failed to decode Cip509: {:?}",
            cip509_1.report()
        );

        let tracking_payment_keys = vec![];

        // TODO: FIXME: The transaction shouldn't be used here.
        let transactions_1 = multi_era_block_1.txs();
        // Forth transaction of this test data contains the CIP509 auxiliary data
        let tx_1 = transactions_1
            .get(3)
            .expect("Failed to get transaction index");
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

        let cip509 = Cip509::new(&multi_era_block_4, 1.into()).unwrap().unwrap();
        assert!(
            !cip509.report().is_problematic(),
            "Failed to decode Cip509: {:?}",
            cip509.report()
        );

        // TODO: FIXME: The transaction shouldn't be used here.
        let transactions_4 = multi_era_block_4.txs();
        // Second transaction of this test data contains the CIP509 auxiliary data
        let tx = transactions_4
            .get(1)
            .expect("Failed to get transaction index");

        // Update the registration chain
        assert!(registration_chain
            .unwrap()
            .update(point_4.clone(), 1, tx, cip509)
            .is_ok());
    }
}
