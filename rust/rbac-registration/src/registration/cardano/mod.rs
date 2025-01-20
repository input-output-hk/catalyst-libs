//! Chain of Cardano registration data

use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use c509_certificate::c509::C509;
use catalyst_types::hashes::Blake2b256Hash;
use ed25519_dalek::VerifyingKey;
use tracing::{error, warn};
use uuid::Uuid;
use x509_cert::certificate::Certificate as X509Certificate;

use crate::cardano::cip509::{
    C509Cert, CertKeyHash, Cip0134UriSet, Cip509, PaymentHistory, PointTxnIdx, RoleData,
    RoleNumber, SimplePublicKeyType, X509DerCert,
};

/// Registration chains.
#[derive(Debug)]
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
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    pub fn new(cip509: Cip509) -> anyhow::Result<Self> {
        let inner = RegistrationChainInner::new(cip509)?;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Update the registration chain.
    ///
    /// # Arguments
    /// - `point` - The point (slot) of the transaction.
    /// - `tx_idx` - The transaction index.
    /// - `cip509` - The CIP509.
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    pub fn update(&self, cip509: Cip509) -> anyhow::Result<Self> {
        let new_inner = self.inner.update(cip509)?;

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
    pub fn x509_certs(&self) -> &HashMap<usize, (PointTxnIdx, X509Certificate)> {
        &self.inner.x509_certs
    }

    /// Get the map of index in array to point, transaction index, and c509 certificate.
    #[must_use]
    pub fn c509_certs(&self) -> &HashMap<usize, (PointTxnIdx, C509)> {
        &self.inner.c509_certs
    }

    /// Get the map of index in array to point, transaction index, and public key.
    #[must_use]
    pub fn simple_keys(&self) -> &HashMap<usize, (PointTxnIdx, VerifyingKey)> {
        &self.inner.simple_keys
    }

    /// Get a list of revocations.
    #[must_use]
    pub fn revocations(&self) -> &[(PointTxnIdx, CertKeyHash)] {
        &self.inner.revocations
    }

    /// Get the map of role number to point, transaction index, and role data.
    #[must_use]
    pub fn role_data(&self) -> &HashMap<RoleNumber, (PointTxnIdx, RoleData)> {
        &self.inner.role_data
    }

    /// Get the map of tracked payment keys to its history.
    #[must_use]
    pub fn tracking_payment_history(&self) -> &PaymentHistory {
        &self.inner.payment_history
    }
}

/// Inner structure of registration chain.
#[derive(Debug, Clone)]
struct RegistrationChainInner {
    /// The current transaction ID hash (32 bytes)
    current_tx_id_hash: Blake2b256Hash,
    /// List of purpose for this registration chain
    purpose: Vec<Uuid>,

    // RBAC
    /// Map of index in array to point, transaction index, and x509 certificate.
    x509_certs: HashMap<usize, (PointTxnIdx, X509Certificate)>,
    /// Map of index in array to point, transaction index, and c509 certificate.
    c509_certs: HashMap<usize, (PointTxnIdx, C509)>,
    /// A set of URIs contained in both x509 and c509 certificates.
    certificate_uris: Cip0134UriSet,
    /// Map of index in array to point, transaction index, and public key.
    simple_keys: HashMap<usize, (PointTxnIdx, VerifyingKey)>,
    /// List of point, transaction index, and certificate key hash.
    revocations: Vec<(PointTxnIdx, CertKeyHash)>,

    // Role
    /// Map of role number to point, transaction index, and role data.
    role_data: HashMap<RoleNumber, (PointTxnIdx, RoleData)>,
    /// Map of tracked payment key to its history.
    payment_history: PaymentHistory,
}

impl RegistrationChainInner {
    /// Create a new instance of registration chain.
    /// The first new value should be the chain root.
    ///
    /// # Arguments
    /// - `cip509` - The CIP509.
    ///
    /// # Errors
    ///
    /// Returns an error if data is invalid
    fn new(cip509: Cip509) -> anyhow::Result<Self> {
        // Should be chain root, return immediately if not
        if cip509.previous_transaction().is_some() {
            bail!("Invalid chain root, previous transaction ID should be None.");
        }

        let point_tx_idx = cip509.origin().clone();
        let current_tx_id_hash = cip509.txn_hash();
        let (purpose, registration, payment_history) = match cip509.consume() {
            Ok(v) => v,
            Err(e) => {
                let error = format!("Invalid Cip509: {e:?}");
                error!(error);
                bail!(error);
            },
        };

        let purpose = vec![purpose];
        let certificate_uris = registration.certificate_uris;
        let x509_certs = chain_root_x509_certs(registration.x509_certs, &point_tx_idx);
        let c509_certs = chain_root_c509_certs(registration.c509_certs, &point_tx_idx);
        let simple_keys = chain_root_public_keys(registration.pub_keys, &point_tx_idx);
        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        let role_data = chain_root_role_data(registration.role_data, &point_tx_idx);

        Ok(Self {
            current_tx_id_hash,
            purpose,
            x509_certs,
            c509_certs,
            certificate_uris,
            simple_keys,
            revocations,
            role_data,
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
    fn update(&self, cip509: Cip509) -> anyhow::Result<Self> {
        let mut new_inner = self.clone();

        let Some(prv_tx_id) = cip509.previous_transaction() else {
            bail!("Empty previous transaction ID");
        };
        // Previous transaction ID in the CIP509 should equal to the current transaction ID
        // or else it is not a part of the chain
        if prv_tx_id == self.current_tx_id_hash {
            // Update the current transaction ID hash
            new_inner.current_tx_id_hash = cip509.txn_hash();
        } else {
            bail!("Invalid previous transaction ID, not a part of this registration chain");
        }

        let point_tx_idx = cip509.origin().clone();
        let (purpose, registration, payment_history) = match cip509.consume() {
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

        new_inner.certificate_uris = new_inner.certificate_uris.update(&registration);
        new_inner.payment_history.extend(payment_history);
        update_x509_certs(&mut new_inner, registration.x509_certs, &point_tx_idx);
        update_c509_certs(&mut new_inner, registration.c509_certs, &point_tx_idx);
        update_public_keys(&mut new_inner, registration.pub_keys, &point_tx_idx);

        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        // Revocation list should be appended
        new_inner.revocations.extend(revocations);

        update_role_data(&mut new_inner, registration.role_data, &point_tx_idx);

        Ok(new_inner)
    }
}

/// Process x509 certificate for chain root.
fn chain_root_x509_certs(
    x509_certs: Vec<X509DerCert>, point_tx_idx: &PointTxnIdx,
) -> HashMap<usize, (PointTxnIdx, X509Certificate)> {
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
    new_inner: &mut RegistrationChainInner, x509_certs: Vec<X509DerCert>,
    point_tx_idx: &PointTxnIdx,
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
    c509_certs: Vec<C509Cert>, point_tx_idx: &PointTxnIdx,
) -> HashMap<usize, (PointTxnIdx, C509)> {
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
    new_inner: &mut RegistrationChainInner, c509_certs: Vec<C509Cert>, point_tx_idx: &PointTxnIdx,
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
    pub_keys: Vec<SimplePublicKeyType>, point_tx_idx: &PointTxnIdx,
) -> HashMap<usize, (PointTxnIdx, VerifyingKey)> {
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
    point_tx_idx: &PointTxnIdx,
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
    revocation_list: Vec<CertKeyHash>, point_tx_idx: &PointTxnIdx,
) -> Vec<(PointTxnIdx, CertKeyHash)> {
    let mut revocations = Vec::new();
    for item in revocation_list {
        revocations.push((point_tx_idx.clone(), item.clone()));
    }
    revocations
}

/// Process the role data for chain root.
fn chain_root_role_data(
    role_data: HashMap<RoleNumber, RoleData>, point_tx_idx: &PointTxnIdx,
) -> HashMap<RoleNumber, (PointTxnIdx, RoleData)> {
    role_data
        .into_iter()
        .map(|(number, data)| (number, (point_tx_idx.clone(), data)))
        .collect()
}

/// Update the role data in the registration chain.
fn update_role_data(
    inner: &mut RegistrationChainInner, role_set: HashMap<RoleNumber, RoleData>,
    point_tx_idx: &PointTxnIdx,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::test;

    #[test]
    fn multiple_registrations() {
        let data = test::block_1();
        let registration = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        data.assert_valid(&registration);

        // Create a chain with the first registration.
        let chain = RegistrationChain::new(registration).unwrap();
        assert_eq!(chain.purpose(), &[data.purpose]);
        assert_eq!(1, chain.x509_certs().len());
        let origin = &chain.x509_certs().get(&0).unwrap().0;
        assert_eq!(origin.point().slot_or_default(), data.slot);
        assert_eq!(origin.txn_index(), data.txn_index);

        // Try to add an invalid registration.
        let data = test::block_2();
        let registration = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        assert!(registration.report().is_problematic());

        let error = chain.update(registration).unwrap_err();
        let error = format!("{error:?}");
        assert!(
            error.contains("Invalid previous transaction ID"),
            "{}",
            error
        );

        // Add the second registration.
        let data = test::block_4();
        let registration = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        data.assert_valid(&registration);
        let update = chain.update(registration).unwrap();

        // Current tx hash should be equal to the hash from block 4.
        assert_eq!(update.current_tx_id_hash(), data.txn_hash);
        assert!(update.role_data().contains_key(&data.role));
    }
}
