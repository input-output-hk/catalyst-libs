//! Chain of Cardano registration data

use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use c509_certificate::c509::C509;
use cardano_blockchain_types::TransactionId;
use catalyst_types::{id_uri::IdUri, uuid::UuidV4};
use ed25519_dalek::VerifyingKey;
use tracing::{error, warn};
use x509_cert::certificate::Certificate as X509Certificate;

use crate::cardano::cip509::{
    C509Cert, CertKeyHash, Cip0134UriSet, Cip509, PaymentHistory, PointData, PointTxnIdx, RoleData,
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

    /// Returns a Catalyst ID.
    #[must_use]
    pub fn catalyst_id(&self) -> &IdUri {
        &self.inner.catalyst_id
    }

    /// Get the current transaction ID hash.
    #[must_use]
    pub fn current_tx_id_hash(&self) -> TransactionId {
        self.inner.current_tx_id_hash
    }

    /// Get a list of purpose for this registration chain.
    #[must_use]
    pub fn purpose(&self) -> &[UuidV4] {
        &self.inner.purpose
    }

    /// Get the map of index in array to list of point + transaction index, and x509
    /// certificate.
    #[must_use]
    pub fn x509_certs(&self) -> &HashMap<usize, Vec<PointData<Option<X509Certificate>>>> {
        &self.inner.x509_certs
    }

    /// Get the map of index in array to list of point + transaction index, and c509
    /// certificate.
    #[must_use]
    pub fn c509_certs(&self) -> &HashMap<usize, Vec<PointData<Option<C509>>>> {
        &self.inner.c509_certs
    }

    /// Get the map of index in array to list of point + transaction index, and public
    /// key.
    #[must_use]
    pub fn simple_keys(&self) -> &HashMap<usize, Vec<PointData<Option<VerifyingKey>>>> {
        &self.inner.simple_keys
    }

    /// Get a list of point + transaction index and revocation.
    #[must_use]
    pub fn revocations(&self) -> &[PointData<CertKeyHash>] {
        &self.inner.revocations
    }

    /// Get the map of latest update role number to point + transaction index, and role
    /// data.
    #[must_use]
    pub fn role_data(&self) -> &HashMap<RoleNumber, PointData<RoleData>> {
        &self.inner.role_data
    }

    /// Get the map of role number to list of point + transaction index, and role data.
    #[must_use]
    pub fn all_role_data(&self) -> &HashMap<RoleNumber, Vec<PointData<RoleData>>> {
        &self.inner.all_role_data
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
    /// A Catalyst ID.
    catalyst_id: IdUri,
    /// The current transaction ID hash (32 bytes)
    current_tx_id_hash: TransactionId,
    /// List of purpose for this registration chain
    purpose: Vec<UuidV4>,

    // RBAC
    /// Map of index in array to list of point + transaction index, and optional x509
    /// certificate. If X509 is None, it means the certificate is deleted.
    x509_certs: HashMap<usize, Vec<PointData<Option<X509Certificate>>>>,
    /// Map of index in array to list of point + transaction index, and optional c509
    /// certificate. If C509 is None, it means the certificate is deleted.
    c509_certs: HashMap<usize, Vec<PointData<Option<C509>>>>,
    /// A set of URIs contained in both x509 and c509 certificates.
    certificate_uris: Cip0134UriSet,
    /// Map of index in array to list of point + transaction index, and public key.
    /// If key is None, it means the key is deleted.
    simple_keys: HashMap<usize, Vec<PointData<Option<VerifyingKey>>>>,
    /// List of point + transaction index, and certificate key hash.
    revocations: Vec<PointData<CertKeyHash>>,

    // Role
    /// Map of role number to list point + transaction index, and role data.
    /// Cannot use the latest one since role data can be partially updated.
    all_role_data: HashMap<RoleNumber, Vec<PointData<RoleData>>>,
    /// The latest update of role data associated to the role number.
    role_data: HashMap<RoleNumber, PointData<RoleData>>,
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
        let Some(catalyst_id) = cip509.catalyst_id().cloned() else {
            bail!("Invalid chain root, catalyst id should be present.");
        };

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
        let mut x509_certs = HashMap::new();
        update_x509_certs(&mut x509_certs, registration.x509_certs, &point_tx_idx);
        let mut c509_certs = HashMap::new();
        update_c509_certs(&mut c509_certs, registration.c509_certs, &point_tx_idx);
        let mut simple_keys = HashMap::new();
        update_public_keys(&mut simple_keys, registration.pub_keys, &point_tx_idx);
        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);

        // Role data
        let mut all_role_data = HashMap::new();
        let mut role_data = HashMap::new();
        for (number, data) in registration.role_data {
            let point_data = PointData::new(point_tx_idx.clone(), data);
            all_role_data.insert(number, vec![point_data.clone()]);
            role_data.insert(number, point_data);
        }

        Ok(Self {
            catalyst_id,
            current_tx_id_hash,
            purpose,
            x509_certs,
            c509_certs,
            certificate_uris,
            simple_keys,
            revocations,
            all_role_data,
            role_data,
            payment_history,
        })
    }

    /// Update the registration chain.
    ///
    /// # Arguments
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
        if cip509.catalyst_id().is_some() {
            bail!("Catalyst id should be present only for chain root registration.");
        }
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
        update_x509_certs(
            &mut new_inner.x509_certs,
            registration.x509_certs,
            &point_tx_idx,
        );
        update_c509_certs(
            &mut new_inner.c509_certs,
            registration.c509_certs,
            &point_tx_idx,
        );
        update_public_keys(
            &mut new_inner.simple_keys,
            registration.pub_keys,
            &point_tx_idx,
        );

        let revocations = revocations_list(registration.revocation_list, &point_tx_idx);
        // Revocation list should be appended
        new_inner.revocations.extend(revocations);

        update_role_data(&mut new_inner, registration.role_data, &point_tx_idx);

        Ok(new_inner)
    }
}

/// Update x509 certificates in the registration chain.
fn update_x509_certs(
    x509_cert_map: &mut HashMap<usize, Vec<PointData<Option<X509Certificate>>>>,
    x509_certs: Vec<X509DerCert>, point_tx_idx: &PointTxnIdx,
) {
    for (idx, cert) in x509_certs.into_iter().enumerate() {
        match cert {
            // Unchanged to that index
            X509DerCert::Undefined => {
                if let Some(cert_vec) = x509_cert_map.get_mut(&idx) {
                    // Get the previous (last) one since the certificate is unchanged
                    if let Some(last_cert) = cert_vec.last() {
                        cert_vec.push(PointData::new(
                            point_tx_idx.clone(),
                            last_cert.data().clone(),
                        ));
                    }
                }
            },
            // Delete the certificate, set to none
            X509DerCert::Deleted => {
                x509_cert_map
                    .entry(idx)
                    .or_default()
                    .push(PointData::new(point_tx_idx.clone(), None));
            },
            // Add the new certificate
            X509DerCert::X509Cert(cert) => {
                x509_cert_map
                    .entry(idx)
                    .or_default()
                    .push(PointData::new(point_tx_idx.clone(), Some(*cert)));
            },
        }
    }
}

/// Update c509 certificates in the registration chain.
fn update_c509_certs(
    c509_cert_map: &mut HashMap<usize, Vec<PointData<Option<C509>>>>, c509_certs: Vec<C509Cert>,
    point_tx_idx: &PointTxnIdx,
) {
    for (idx, cert) in c509_certs.into_iter().enumerate() {
        match cert {
            // Unchanged to that index
            C509Cert::Undefined => {
                if let Some(cert_vec) = c509_cert_map.get_mut(&idx) {
                    // Get the previous (last) one since the certificate is unchanged
                    if let Some(last_cert) = cert_vec.last() {
                        cert_vec.push(PointData::new(
                            point_tx_idx.clone(),
                            last_cert.data().clone(),
                        ));
                    }
                }
            },
            // Delete the certificate, set to none
            C509Cert::Deleted => {
                c509_cert_map
                    .entry(idx)
                    .or_default()
                    .push(PointData::new(point_tx_idx.clone(), None));
            },
            // Certificate reference
            C509Cert::C509CertInMetadatumReference(_) => {
                warn!("Unsupported C509CertInMetadatumReference");
            },
            // Add the new certificate
            C509Cert::C509Certificate(cert) => {
                c509_cert_map
                    .entry(idx)
                    .or_default()
                    .push(PointData::new(point_tx_idx.clone(), Some(*cert)));
            },
        }
    }
}

/// Update public keys in the registration chain.
fn update_public_keys(
    pub_key_map: &mut HashMap<usize, Vec<PointData<Option<VerifyingKey>>>>,
    pub_keys: Vec<SimplePublicKeyType>, point_tx_idx: &PointTxnIdx,
) {
    for (idx, cert) in pub_keys.into_iter().enumerate() {
        match cert {
            // Unchanged to that index
            SimplePublicKeyType::Undefined => {
                if let Some(key_vec) = pub_key_map.get_mut(&idx) {
                    // Get the previous (last) one since the certificate is unchanged
                    if let Some(last_key) = key_vec.last() {
                        key_vec.push(PointData::new(point_tx_idx.clone(), *last_key.data()));
                    }
                }
            },
            // Delete the certificate, set to none
            SimplePublicKeyType::Deleted => {
                pub_key_map
                    .entry(idx)
                    .or_default()
                    .push(PointData::new(point_tx_idx.clone(), None));
            },
            // Add the new public key
            SimplePublicKeyType::Ed25519(key) => {
                pub_key_map
                    .entry(idx)
                    .or_default()
                    .push(PointData::new(point_tx_idx.clone(), Some(key)));
            },
        }
    }
}

/// Process the revocation list.
fn revocations_list(
    revocation_list: Vec<CertKeyHash>, point_tx_idx: &PointTxnIdx,
) -> Vec<PointData<CertKeyHash>> {
    let mut revocations = Vec::new();
    for item in revocation_list {
        let point_data = PointData::new(point_tx_idx.clone(), item.clone());
        revocations.push(point_data);
    }
    revocations
}

/// Update the role data in the registration chain.
fn update_role_data(
    inner: &mut RegistrationChainInner, role_set: HashMap<RoleNumber, RoleData>,
    point_tx_idx: &PointTxnIdx,
) {
    for (number, mut data) in role_set {
        // Update the record of role data first
        inner
            .all_role_data
            .entry(number)
            .or_default()
            .push(PointData::new(point_tx_idx.clone(), data.clone()));

        // If there is new role singing key, use it, else use the old one
        if data.signing_key().is_none() {
            let signing_key = inner
                .role_data
                .get(&number)
                .and_then(|pd| pd.data().signing_key())
                .cloned();
            data.set_signing_key(signing_key);
        }

        // If there is new role encryption key, use it, else use the old one
        if data.encryption_key().is_none() {
            let encryption_key = inner
                .role_data
                .get(&number)
                .and_then(|pd| pd.data().encryption_key())
                .cloned();
            data.set_encryption_key(encryption_key);
        }

        // Map of role number to point and role data
        // Note that new role data will overwrite the old one
        inner
            .role_data
            .insert(number, PointData::new(point_tx_idx.clone(), data));
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
        let origin = &chain.x509_certs().get(&0).unwrap().first().unwrap();
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
        // There is only 1 update to role 0 data
        assert_eq!(
            update
                .all_role_data()
                .get(&RoleNumber::ROLE_0)
                .unwrap()
                .len(),
            1
        );
        // There is only 1 update to role 4 data
        assert_eq!(
            update
                .all_role_data()
                .get(&RoleNumber::from(4))
                .unwrap()
                .len(),
            1
        );
        // x509 certificates update on 2 index
        assert_eq!(update.x509_certs().len(), 2);
    }
}
