//! Chain of Cardano registration data

mod update_rbac;

use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use c509_certificate::c509::C509;
use cardano_blockchain_types::TransactionId;
use catalyst_types::{id_uri::IdUri, uuid::UuidV4};
use ed25519_dalek::VerifyingKey;
use tracing::error;
use update_rbac::{
    revocations_list, update_c509_certs, update_public_keys, update_role_data, update_x509_certs,
};
use x509_cert::certificate::Certificate as X509Certificate;

use crate::cardano::cip509::{
    CertKeyHash, CertOrPk, Cip0134UriSet, Cip509, PaymentHistory, PointData, RoleData,
    RoleDataRecord, RoleNumber,
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

    /// Get the map of role number to role data record where each field in role data
    /// record has it own record of its value and its associated point and transaction
    /// index.
    #[must_use]
    pub fn role_data_record(&self) -> &HashMap<RoleNumber, RoleDataRecord> {
        &self.inner.role_data_record
    }

    /// Get the map of role number to list of history data of point + transaction index,
    /// and role data.
    #[must_use]
    pub fn role_data_history(&self) -> &HashMap<RoleNumber, Vec<PointData<RoleData>>> {
        &self.inner.role_data_history
    }

    /// Get the map of tracked payment keys to its history.
    #[must_use]
    pub fn tracking_payment_history(&self) -> &PaymentHistory {
        &self.inner.payment_history
    }

    /// Get the latest signing public key for a role.
    /// Returns the public key and the rotation,`None` if not found.
    #[must_use]
    pub fn get_latest_signing_pk_for_role(
        &self, role: &RoleNumber,
    ) -> Option<(VerifyingKey, usize)> {
        self.inner.role_data_record.get(role).and_then(|rdr| {
            rdr.signing_keys().last().and_then(|key| {
                key.data()
                    .extract_pk()
                    .map(|pk| (pk, rdr.signing_keys().len().saturating_sub(1)))
            })
        })
    }

    /// Get the latest encryption public key for a role.
    /// Returns the public key and the rotation, `None` if not found.
    #[must_use]
    pub fn get_latest_encryption_pk_for_role(
        &self, role: &RoleNumber,
    ) -> Option<(VerifyingKey, usize)> {
        self.inner.role_data_record.get(role).and_then(|rdr| {
            rdr.encryption_keys().last().and_then(|key| {
                key.data()
                    .extract_pk()
                    .map(|pk| (pk, rdr.encryption_keys().len().saturating_sub(1)))
            })
        })
    }

    /// Get signing public key for a role with given rotation.
    /// Returns the public key, `None` if not found.
    #[must_use]
    pub fn get_signing_pk_for_role_at_rotation(
        &self, role: &RoleNumber, rotation: usize,
    ) -> Option<VerifyingKey> {
        self.inner.role_data_record.get(role).and_then(|rdr| {
            rdr.signing_key_from_rotation(rotation)
                .and_then(CertOrPk::extract_pk)
        })
    }

    /// Get encryption public key for a role with given rotation.
    /// Returns the public key, `None` if not found.
    #[must_use]
    pub fn get_encryption_pk_for_role_at_rotation(
        &self, role: &RoleNumber, rotation: usize,
    ) -> Option<VerifyingKey> {
        self.inner.role_data_record.get(role).and_then(|rdr| {
            rdr.encryption_key_from_rotation(rotation)
                .and_then(CertOrPk::extract_pk)
        })
    }

    /// Get signing key X509 certificate, C509 certificate or public key for a role with
    /// given rotation.
    #[must_use]
    pub fn get_singing_key_cert_or_key_for_role_at_rotation(
        &self, role: &RoleNumber, rotation: usize,
    ) -> Option<&CertOrPk> {
        self.inner
            .role_data_record
            .get(role)
            .and_then(|rdr| rdr.signing_key_from_rotation(rotation))
    }

    /// Get encryption key X509 certificate, C509 certificate or public key for a role
    /// with given rotation.
    #[must_use]
    pub fn get_encryption_key_cert_or_key_for_role_at_rotation(
        &self, role: &RoleNumber, rotation: usize,
    ) -> Option<&CertOrPk> {
        self.inner
            .role_data_record
            .get(role)
            .and_then(|rdr| rdr.encryption_key_from_rotation(rotation))
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
    /// Record history of the whole role data in point in time.
    role_data_history: HashMap<RoleNumber, Vec<PointData<RoleData>>>,
    /// Map of role number role data record where each field in role data record
    /// has it own record of its value and its associated point and transaction index.
    role_data_record: HashMap<RoleNumber, RoleDataRecord>,
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
        let certificate_uris = registration.certificate_uris.clone();
        let mut x509_certs = HashMap::new();
        update_x509_certs(
            &mut x509_certs,
            registration.x509_certs.clone(),
            &point_tx_idx,
        );
        let mut c509_certs = HashMap::new();
        update_c509_certs(
            &mut c509_certs,
            registration.c509_certs.clone(),
            &point_tx_idx,
        );
        let mut simple_keys = HashMap::new();
        update_public_keys(
            &mut simple_keys,
            registration.pub_keys.clone(),
            &point_tx_idx,
        );
        let revocations = revocations_list(registration.revocation_list.clone(), &point_tx_idx);

        // Role data
        let mut role_data_history = HashMap::new();
        let mut role_data_record = HashMap::new();

        update_role_data(
            &registration,
            &mut role_data_history,
            &mut role_data_record,
            &point_tx_idx,
        );

        Ok(Self {
            catalyst_id,
            current_tx_id_hash,
            purpose,
            x509_certs,
            c509_certs,
            certificate_uris,
            simple_keys,
            revocations,
            role_data_history,
            role_data_record,
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
            registration.x509_certs.clone(),
            &point_tx_idx,
        );
        update_c509_certs(
            &mut new_inner.c509_certs,
            registration.c509_certs.clone(),
            &point_tx_idx,
        );
        update_public_keys(
            &mut new_inner.simple_keys,
            registration.pub_keys.clone(),
            &point_tx_idx,
        );

        let revocations = revocations_list(registration.revocation_list.clone(), &point_tx_idx);
        // Revocation list should be appended
        new_inner.revocations.extend(revocations);

        update_role_data(
            &registration,
            &mut new_inner.role_data_history,
            &mut new_inner.role_data_record,
            &point_tx_idx,
        );

        Ok(new_inner)
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
        assert!(update.role_data_record().contains_key(&data.role));

        // There is only 1 update to role 0 data
        assert_eq!(
            update
                .role_data_history()
                .get(&RoleNumber::ROLE_0)
                .unwrap()
                .len(),
            1
        );
        // There is only 1 update to role 4 data
        assert_eq!(
            update
                .role_data_history()
                .get(&RoleNumber::from(4))
                .unwrap()
                .len(),
            1
        );

        let role_0_data = update.role_data_record().get(&RoleNumber::ROLE_0).unwrap();
        assert_eq!(role_0_data.signing_keys().len(), 1);
        assert_eq!(role_0_data.encryption_keys().len(), 0);
        assert_eq!(role_0_data.payment_keys().len(), 1);
        assert_eq!(role_0_data.extended_data().len(), 1);

        let role_4_data = update.role_data_record().get(&RoleNumber::from(4)).unwrap();
        assert_eq!(role_4_data.signing_keys().len(), 1);
        assert_eq!(role_4_data.encryption_keys().len(), 0);
        assert_eq!(role_4_data.payment_keys().len(), 1);
        assert_eq!(role_4_data.extended_data().len(), 1);

        // x509 certificates update on 2 index
        assert_eq!(update.x509_certs().len(), 2);

        let (_k, r) = update
            .get_latest_signing_pk_for_role(&RoleNumber::ROLE_0)
            .unwrap();
        assert_eq!(r, 0);
        assert!(update
            .get_latest_encryption_pk_for_role(&RoleNumber::from(4))
            .is_none());
        assert!(update
            .get_signing_pk_for_role_at_rotation(&RoleNumber::ROLE_0, 2)
            .is_none());
        assert!(update
            .get_encryption_pk_for_role_at_rotation(&RoleNumber::from(4), 0)
            .is_none());
        assert!(update
            .get_singing_key_cert_or_key_for_role_at_rotation(&RoleNumber::ROLE_0, 0)
            .is_some());
        assert!(update
            .get_encryption_key_cert_or_key_for_role_at_rotation(&RoleNumber::from(4), 3)
            .is_none());
    }
}
