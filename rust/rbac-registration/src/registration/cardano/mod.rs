//! Chain of Cardano registration data

mod update_rbac;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Context;
use c509_certificate::c509::C509;
use cardano_blockchain_types::{Point, StakeAddress, TxnIndex, hashes::TransactionId};
use catalyst_types::{
    catalyst_id::{CatalystId, key_rotation::KeyRotation, role_index::RoleId},
    conversion::zero_out_last_n_bytes,
    problem_report::ProblemReport,
    uuid::UuidV4,
};
use ed25519_dalek::{Signature, VerifyingKey};
use update_rbac::{
    revocations_list, update_c509_certs, update_public_keys, update_role_data, update_x509_certs,
};
use x509_cert::certificate::Certificate as X509Certificate;

use crate::cardano::{
    cip509::{
        CertKeyHash, CertOrPk, Cip0134UriSet, Cip509, PaymentHistory, PointData, PointTxnIdx,
        RoleData, RoleDataRecord, ValidationSignature,
    },
    provider::RbacChainsProvider,
};

/// Registration chains.
///
/// This structure uses [`Arc`] internally, so it is cheap to clone.
#[derive(Debug, Clone)]
pub struct RegistrationChain {
    /// Inner part of the registration chain.
    inner: Arc<RegistrationChainInner>,
}

impl RegistrationChain {
    /// Attempts to initialize a new RBAC registration chain
    /// from a given CIP-509 registration, ensuring uniqueness of Catalyst ID, stake
    /// addresses, and associated public keys.
    ///
    /// Returns `None` if the `cip509` is invalid by any reason, properly updating
    /// `cip509.report()`.
    ///
    /// # Errors
    ///  - Propagates any I/O or provider-level errors encountered while checking key
    ///    ownership (e.g., database lookup failures).
    pub async fn new(
        cip509: &Cip509,
        provider: &impl RbacChainsProvider,
    ) -> anyhow::Result<Option<Self>> {
        let Some(new_chain) = Self::new_stateless(cip509) else {
            return Ok(None);
        };

        // Verify that a Catalyst ID of this chain is unique.
        {
            let cat_id = new_chain.catalyst_id();
            if provider.is_chain_known(cat_id).await? {
                cip509.report().functional_validation(
            &format!("{} is already used", cat_id.as_short_id()),
                "It isn't allowed to use same Catalyst ID (certificate subject public key) in multiple registration chains",
                    );
            }

            check_signing_public_key(cat_id, cip509, provider).await?;
        }

        if cip509.report().is_problematic() {
            return Ok(None);
        }

        Ok(Some(new_chain))
    }

    /// Create a new instance of registration chain.
    /// The first new value should be the chain root.
    ///
    /// Returns `None` if the `cip509` is invalid by any reason, properly updating
    /// `cip509.report()`.
    #[must_use]
    pub fn new_stateless(cip509: &Cip509) -> Option<Self> {
        let inner = RegistrationChainInner::new(cip509)?;

        Some(Self {
            inner: Arc::new(inner),
        })
    }

    /// Attempts to update an existing RBAC registration chain
    /// with a new CIP-509 registration, validating address and key usage consistency.
    ///
    /// Returns `None` if the `cip509` is invalid by any reason, properly updating
    /// `cip509.report()`.
    ///
    /// # Errors
    ///  - Propagates any I/O or provider-level errors encountered while checking key
    ///    ownership (e.g., database lookup failures).
    pub async fn update(
        &self,
        cip509: &Cip509,
        provider: &impl RbacChainsProvider,
    ) -> anyhow::Result<Option<Self>> {
        let Some(new_chain) = self.update_stateless(cip509) else {
            return Ok(None);
        };

        // Check that addresses from the new registration aren't used in other chains.
        let previous_stake_addresses = self.stake_addresses();
        let reg_stake_addresses = cip509.stake_addresses();
        let new_stake_addresses: Vec<_> = reg_stake_addresses
            .difference(&previous_stake_addresses)
            .collect();
        for address in &new_stake_addresses {
            if provider.is_stake_address_used(address).await? {
                cip509.report().functional_validation(
                        &format!("{address} stake address is already used"),
                        "It isn't allowed to use same stake address in multiple registration chains, if its not a new chain",
                    );
            }
        }

        check_signing_public_key(self.catalyst_id(), cip509, provider).await?;

        if cip509.report().is_problematic() {
            Ok(None)
        } else {
            Ok(Some(new_chain))
        }
    }

    /// Update the registration chain.
    ///
    /// Returns `None` if the `cip509` is invalid by any reason, properly updating
    /// `cip509.report()`.
    #[must_use]
    pub fn update_stateless(
        &self,
        cip509: &Cip509,
    ) -> Option<Self> {
        let latest_signing_pk = self.get_latest_signing_public_key_for_role(RoleId::Role0);
        let new_inner = if let Some((signing_pk, _)) = latest_signing_pk {
            self.inner.update(cip509, signing_pk)?
        } else {
            cip509.report().missing_field(
                "latest signing key for role 0",
                "cannot perform signature validation during Registration Chain update",
            );
            return None;
        };
        Some(Self {
            inner: Arc::new(new_inner),
        })
    }

    /// Returns a Catalyst ID.
    #[must_use]
    pub fn catalyst_id(&self) -> &CatalystId {
        &self.inner.catalyst_id
    }

    /// Get the current transaction ID hash.
    #[must_use]
    pub fn current_tx_id_hash(&self) -> TransactionId {
        *self.inner.current_tx_id_hash.data()
    }

    /// Returns a point (slot) of the latest transaction in the registration chain.
    #[must_use]
    pub fn current_point(&self) -> &Point {
        self.inner.current_tx_id_hash.point()
    }

    /// Returns an index of the latest transaction in the registration chain.
    #[must_use]
    pub fn current_txn_index(&self) -> TxnIndex {
        self.inner.current_tx_id_hash.txn_index()
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
    pub fn role_data_record(&self) -> &HashMap<RoleId, RoleDataRecord> {
        &self.inner.role_data_record
    }

    /// Get the map of role number to list of history data of point + transaction index,
    /// and role data.
    #[must_use]
    pub fn role_data_history(&self) -> &HashMap<RoleId, Vec<PointData<RoleData>>> {
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
    pub fn get_latest_signing_public_key_for_role(
        &self,
        role: RoleId,
    ) -> Option<(VerifyingKey, KeyRotation)> {
        self.inner.get_latest_signing_public_key_for_role(role)
    }

    /// Get the latest encryption public key for a role.
    /// Returns the public key and the rotation, `None` if not found.
    #[must_use]
    pub fn get_latest_encryption_public_key_for_role(
        &self,
        role: RoleId,
    ) -> Option<(VerifyingKey, KeyRotation)> {
        self.inner.get_latest_encryption_public_key_for_role(role)
    }

    /// Get signing public key for a role with given rotation.
    /// Returns the public key, `None` if not found.
    #[must_use]
    pub fn get_signing_pk_for_role_at_rotation(
        &self,
        role: &RoleId,
        rotation: &KeyRotation,
    ) -> Option<VerifyingKey> {
        self.inner.role_data_record.get(role).and_then(|rdr| {
            rdr.signing_key_from_rotation(rotation)
                .and_then(CertOrPk::extract_public_key)
        })
    }

    /// Get encryption public key for a role with given rotation.
    /// Returns the public key, `None` if not found.
    #[must_use]
    pub fn get_encryption_pk_for_role_at_rotation(
        &self,
        role: &RoleId,
        rotation: &KeyRotation,
    ) -> Option<VerifyingKey> {
        self.inner.role_data_record.get(role).and_then(|rdr| {
            rdr.encryption_key_from_rotation(rotation)
                .and_then(CertOrPk::extract_public_key)
        })
    }

    /// Get signing key X509 certificate, C509 certificate or public key for a role with
    /// given rotation.
    #[must_use]
    pub fn get_singing_key_cert_or_key_for_role_at_rotation(
        &self,
        role: &RoleId,
        rotation: &KeyRotation,
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
        &self,
        role: &RoleId,
        rotation: &KeyRotation,
    ) -> Option<&CertOrPk> {
        self.inner
            .role_data_record
            .get(role)
            .and_then(|rdr| rdr.encryption_key_from_rotation(rotation))
    }

    /// Returns most recent URIs contained from both x509 and c509 certificates.
    #[must_use]
    pub fn certificate_uris(&self) -> &Cip0134UriSet {
        &self.inner.certificate_uris
    }

    /// Returns all stake addresses associated to this chain.
    #[must_use]
    pub fn stake_addresses(&self) -> HashSet<StakeAddress> {
        self.inner.certificate_uris.active_stake_addresses()
    }

    /// Returns history information about stake addresses used in this chain.
    #[must_use]
    pub fn stake_addresses_history(&self) -> HashSet<StakeAddress> {
        todo!()
    }

    /// Returns the latest know applied registration's `PointTxnIdx`.
    #[must_use]
    pub fn latest_applied(&self) -> PointTxnIdx {
        self.inner.latest_applied()
    }
}

/// Inner structure of registration chain.
#[derive(Debug, Clone)]
struct RegistrationChainInner {
    /// A Catalyst ID.
    catalyst_id: CatalystId,
    /// The current transaction ID hash (32 bytes)
    current_tx_id_hash: PointData<TransactionId>,
    /// The latest `PointTxnIdx` of the taken URIs by another registration chains.
    latest_taken_uris_point: Option<PointTxnIdx>,

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
    role_data_history: HashMap<RoleId, Vec<PointData<RoleData>>>,
    /// Map of role number role data record where each field in role data record
    /// has it own record of its value and its associated point and transaction index.
    role_data_record: HashMap<RoleId, RoleDataRecord>,
    /// Map of tracked payment key to its history.
    payment_history: PaymentHistory,
}

impl RegistrationChainInner {
    /// Create a new instance of registration chain.
    /// The first new value should be the chain root.
    ///
    /// # Arguments
    /// - `cip509` - The CIP509.
    #[must_use]
    fn new(cip509: &Cip509) -> Option<Self> {
        let context = "Registration Chain new";
        // Should be chain root, return immediately if not
        if cip509.previous_transaction().is_some() {
            cip509
                .report()
                .invalid_value("previous transaction ID", "None", "Some", context);
        }
        let Some(catalyst_id) = cip509.catalyst_id().cloned() else {
            cip509.report().missing_field("catalyst id", context);
            return None;
        };

        let Some(registration) = cip509.metadata() else {
            cip509.report().missing_field("metadata", context);
            return None;
        };

        // Role data
        let mut role_data_history = HashMap::new();
        let mut role_data_record = HashMap::new();
        let point_tx_idx = cip509.origin().clone();
        update_role_data(
            registration,
            &mut role_data_history,
            &mut role_data_record,
            &point_tx_idx,
        );

        // There should be role 0 since we already check that the chain root (no previous tx id)
        // must contain role 0
        let Some(role0_data) = role_data_record.get(&RoleId::Role0) else {
            cip509.report().missing_field("Role 0", context);
            return None;
        };
        let Some(signing_pk) = role0_data
            .signing_keys()
            .last()
            .and_then(|key| key.data().extract_public_key())
        else {
            cip509
                .report()
                .missing_field("Signing pk for role 0 not found", context);
            return None;
        };

        check_validation_signature(
            cip509.validation_signature(),
            cip509.raw_aux_data(),
            signing_pk,
            cip509.report(),
            context,
        );

        if cip509.txn_inputs_hash().is_none() {
            cip509.report().missing_field("txn inputs hash", context);
        }

        let Some(purpose) = cip509.purpose() else {
            cip509.report().missing_field("purpose", context);
            return None;
        };

        if cip509.report().is_problematic() {
            return None;
        }

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
        let current_tx_id_hash = PointData::new(point_tx_idx, cip509.txn_hash());
        let payment_history = cip509.payment_history().clone();

        Some(Self {
            catalyst_id,
            current_tx_id_hash,
            purpose,
            x509_certs,
            c509_certs,
            certificate_uris,
            latest_taken_uris_point: None,
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
    #[must_use]
    fn update(
        &self,
        cip509: &Cip509,
        signing_pk: VerifyingKey,
    ) -> Option<Self> {
        let context = "Registration Chain update";
        if self.latest_applied().point() >= cip509.origin().point() {
            cip509.report().functional_validation(
                &format!(
                    "The provided registration is earlier {} than the latest applied one {}",
                    cip509.origin().point(),
                    self.current_tx_id_hash.point()
                ),
                "Registrations must be applied in the correct ascending order.",
            );
            return None;
        }

        let mut new_inner = self.clone();

        let Some(prv_tx_id) = cip509.previous_transaction() else {
            if let Some(cat_id) = cip509.catalyst_id() {
                if cat_id == &self.catalyst_id {
                    cip509.report().functional_validation(
                        &format!(
                            "Trying to apply the first registration to the associated {} again",
                            cat_id.as_short_id()
                        ),
                        "It isn't allowed to submit first registration twice",
                    );
                    return None;
                }

                return Some(new_inner.update_cause_another_chain(cip509));
            }
            cip509
                .report()
                .missing_field("previous transaction ID", context);
            return None;
        };

        // Previous transaction ID in the CIP509 should equal to the current transaction ID
        if &prv_tx_id == self.current_tx_id_hash.data() {
            // Perform signature validation
            // This should be done before updating the signing key
            check_validation_signature(
                cip509.validation_signature(),
                cip509.raw_aux_data(),
                signing_pk,
                cip509.report(),
                context,
            );

            // If successful, update the chain current transaction ID hash
            new_inner.current_tx_id_hash =
                PointData::new(cip509.origin().clone(), cip509.txn_hash());
        } else {
            cip509.report().invalid_value(
                "previous transaction ID",
                &format!("{prv_tx_id:?}"),
                &format!("{:?}", self.current_tx_id_hash),
                context,
            );
            return None;
        }

        if cip509.txn_inputs_hash().is_none() {
            cip509.report().missing_field("txn inputs hash", context);
        }

        let Some(purpose) = cip509.purpose() else {
            cip509.report().missing_field("purpose", context);
            return None;
        };
        let Some(registration) = cip509.metadata().cloned() else {
            cip509.report().missing_field("metadata", context);
            return None;
        };

        if cip509.report().is_problematic() {
            return None;
        }

        // Add purpose to the chain, if not already exist
        if !self.purpose.contains(&purpose) {
            new_inner.purpose.push(purpose);
        }

        let point_tx_idx = cip509.origin().clone();

        new_inner.certificate_uris = new_inner.certificate_uris.update(&registration);
        new_inner
            .payment_history
            .extend(cip509.payment_history().clone());
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

        Some(new_inner)
    }

    /// Update the registration chain with the `cip509` associated to another chain.
    /// This is the case when registration for different chain affecting the current one,
    /// by invalidating some data for the current registration chain (stealing stake
    /// addresses etc.).
    ///
    /// The provided `cip509` should be fully validated by another chain before trying to
    /// submit to the current one.
    #[must_use]
    fn update_cause_another_chain(
        mut self,
        cip509: &Cip509,
    ) -> Self {
        if let Some(reg) = cip509.metadata() {
            self.certificate_uris = self.certificate_uris.update_taken_uris(reg);
        }
        self.latest_taken_uris_point = Some(cip509.origin().clone());
        self
    }

    /// Get the latest signing public key for a role.
    /// Returns the public key and the rotation,`None` if not found.
    #[must_use]
    pub fn get_latest_signing_public_key_for_role(
        &self,
        role: RoleId,
    ) -> Option<(VerifyingKey, KeyRotation)> {
        self.role_data_record.get(&role).and_then(|rdr| {
            rdr.signing_keys().last().and_then(|key| {
                let rotation = KeyRotation::from_latest_rotation(rdr.signing_keys());

                key.data().extract_public_key().map(|pk| (pk, rotation))
            })
        })
    }

    /// Get the latest encryption public key for a role.
    /// Returns the public key and the rotation, `None` if not found.
    #[must_use]
    pub fn get_latest_encryption_public_key_for_role(
        &self,
        role: RoleId,
    ) -> Option<(VerifyingKey, KeyRotation)> {
        self.role_data_record.get(&role).and_then(|rdr| {
            rdr.encryption_keys().last().and_then(|key| {
                let rotation = KeyRotation::from_latest_rotation(rdr.encryption_keys());

                key.data().extract_public_key().map(|pk| (pk, rotation))
            })
        })
    }

    /// Returns the latest know applied registration's `PointTxnIdx`.
    #[must_use]
    fn latest_applied(&self) -> PointTxnIdx {
        if let Some(latest_taken_uris_point) = &self.latest_taken_uris_point
            && latest_taken_uris_point.point() > self.current_tx_id_hash.point()
        {
            return latest_taken_uris_point.clone();
        }

        PointTxnIdx::new(
            self.current_tx_id_hash.point().clone(),
            self.current_tx_id_hash.txn_index(),
        )
    }
}

/// Perform a check on the validation signature.
/// The auxiliary data should be sign with the latest signing public key.
fn check_validation_signature(
    validation_signature: Option<&ValidationSignature>,
    raw_aux_data: &[u8],
    signing_pk: VerifyingKey,
    report: &ProblemReport,
    context: &str,
) {
    let context = &format!("Check Validation Signature in {context}");
    // Note that the validation signature can be in the range of 1 - 64 bytes
    // But since we allow only Ed25519, it should be 64 bytes
    let unsigned_aux = zero_out_last_n_bytes(raw_aux_data, Signature::BYTE_SIZE);

    let Some(validation_sig) = validation_signature else {
        report.missing_field("validation signature", context);
        return;
    };

    let Ok(sig) = validation_sig.clone().try_into() else {
        report.conversion_error(
            "validation signature",
            &format!("{validation_sig:?}"),
            "Ed25519 signature",
            context,
        );
        return;
    };

    // Verify the signature using the latest signing public key
    if let Err(e) = signing_pk
        .verify_strict(&unsigned_aux, &sig)
        .with_context(|| {
            report.other("Signature validation failed", context);
            "Signature verification failed"
        })
    {
        report.functional_validation(&format!("Signature validation failed: {e}"), context);
    }
}

/// Checks that a new registration doesn't contain a signing key that was used by any
/// other chain.
async fn check_signing_public_key(
    cat_id: &CatalystId,
    cip509: &Cip509,
    state: &impl RbacChainsProvider,
) -> anyhow::Result<()> {
    for role in cip509.all_roles() {
        if let Some(key) = cip509.signing_public_key_for_role(role)
            && let Some(previous) = state
                .chain_catalyst_id_from_signing_public_key(&key)
                .await?
            && &previous != cat_id
        {
            cip509.report().functional_validation(
                    &format!("An update to {cat_id} registration chain uses the same public key ({key:?}) as {previous} chain"),
                    "It isn't allowed to use signing (certificate subject public) key in different chains",
                    );
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use catalyst_types::catalyst_id::role_index::RoleId;

    use super::*;
    use crate::utils::test;

    #[test]
    fn multiple_registrations() {
        let data = test::block_5();
        let registration = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        data.assert_valid(&registration);

        // Performs only stateless validations
        // Create a chain with the first registration.
        let chain = RegistrationChain::new_stateless(&registration).unwrap();
        assert_eq!(chain.purpose(), &[data.purpose]);
        assert_eq!(1, chain.x509_certs().len());
        let origin = &chain.x509_certs().get(&0).unwrap().first().unwrap();
        assert_eq!(origin.point().slot_or_default(), data.slot);
        assert_eq!(origin.txn_index(), data.txn_index);

        // no encryption key is included for the role
        assert!(
            chain
                .get_encryption_pk_for_role_at_rotation(&RoleId::Role0, &KeyRotation::default())
                .is_none()
        );

        assert!(
            chain
                .get_encryption_key_cert_or_key_for_role_at_rotation(
                    &RoleId::Role0,
                    &KeyRotation::default()
                )
                .is_none()
        );

        // Try to add an invalid registration.
        let data = test::block_2();
        let registration = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        assert!(registration.report().is_problematic());

        let report = registration.report().to_owned();
        assert!(chain.update_stateless(&registration).is_none());
        assert!(report.is_problematic(), "{report:?}");

        // Add the second registration.
        let data = test::block_6();
        let registration = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        data.assert_valid(&registration);
        let update = chain.update_stateless(&registration).unwrap();
        // Current tx hash should be equal to the hash from block 4.
        assert_eq!(update.current_tx_id_hash(), data.txn_hash);
        assert!(update.role_data_record().contains_key(&data.role));
        // Update contains changes to role 0 without adding more roles.
        assert_eq!(update.role_data_record().len(), 1);

        // There are 2 updates to role 0 data.
        assert_eq!(
            update
                .role_data_history()
                .get(&RoleId::Role0)
                .unwrap()
                .len(),
            2
        );

        let role_0_data = update.role_data_record().get(&RoleId::Role0).unwrap();
        assert_eq!(role_0_data.signing_keys().len(), 2);
        assert_eq!(role_0_data.encryption_keys().len(), 0);
        assert_eq!(role_0_data.payment_keys().len(), 2);
        assert_eq!(role_0_data.extended_data().len(), 2);

        let (_k, r) = update
            .get_latest_signing_public_key_for_role(RoleId::Role0)
            .unwrap();
        assert_eq!(r, KeyRotation::from(1));
        assert!(
            update
                .get_signing_pk_for_role_at_rotation(&RoleId::Role0, &KeyRotation::from(2))
                .is_none()
        );
        assert!(
            update
                .get_singing_key_cert_or_key_for_role_at_rotation(
                    &RoleId::Role0,
                    &KeyRotation::from(0)
                )
                .is_some()
        );
    }
}
