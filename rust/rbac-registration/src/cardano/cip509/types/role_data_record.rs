//! Role data record where each role data fields are stored.

use std::collections::HashMap;

use catalyst_types::catalyst_id::key_rotation::KeyRotation;
use pallas::ledger::addresses::ShelleyAddress;

use super::{CertOrPk, PointData, PointTxnIdx};

/// Role data record where each field has it own record of its value and its associated
/// point and transaction index. If a field has key rotation, then the vector index is
/// used. Eg. Accessing key rotation 0 can be done by `signing_keys.first()`
#[derive(Debug, Clone)]
pub struct RoleDataRecord {
    /// List of signing key data and its associated point + tx index .
    /// The vector index is used to indicate the key rotation.
    signing_keys: Vec<PointData<CertOrPk>>,
    /// List of encryption key data and its associated point + tx index
    /// The vector index is used to indicate the key rotation.
    encryption_keys: Vec<PointData<CertOrPk>>,
    /// List of payment key and its associated point + tx index.
    payment_keys: Vec<PointData<ShelleyAddress>>,
    /// List of extended data and its associated point + tx index.
    extended_data: Vec<PointData<HashMap<u8, Vec<u8>>>>,
}

impl RoleDataRecord {
    /// Create a new empty role data record.
    pub(crate) fn new() -> Self {
        Self {
            signing_keys: Vec::new(),
            encryption_keys: Vec::new(),
            payment_keys: Vec::new(),
            extended_data: Vec::new(),
        }
    }

    /// Add a signing key data to the signing key list. The vector index is used to
    /// indicate the key rotation.
    pub(crate) fn add_signing_key(
        &mut self,
        data: CertOrPk,
        point_tx_idx: &PointTxnIdx,
    ) {
        self.signing_keys
            .push(PointData::new(point_tx_idx.clone(), data));
    }

    /// Add an encryption key data to the encryption key list. The vector index is used to
    /// indicate the key rotation.
    pub(crate) fn add_encryption_key(
        &mut self,
        data: CertOrPk,
        point_tx_idx: &PointTxnIdx,
    ) {
        self.encryption_keys
            .push(PointData::new(point_tx_idx.clone(), data));
    }

    /// Add a payment key to the payment key list.
    pub(crate) fn add_payment_key(
        &mut self,
        data: PointData<ShelleyAddress>,
    ) {
        self.payment_keys.push(data);
    }

    /// Add extended data to the extended data list.
    pub(crate) fn add_extended_data(
        &mut self,
        data: PointData<HashMap<u8, Vec<u8>>>,
    ) {
        self.extended_data.push(data);
    }

    /// Get the list of signing keys associated with this role.
    #[must_use]
    pub fn signing_keys(&self) -> &Vec<PointData<CertOrPk>> {
        &self.signing_keys
    }

    /// Get the list of encryption keys associated with this role.
    #[must_use]
    pub fn encryption_keys(&self) -> &Vec<PointData<CertOrPk>> {
        &self.encryption_keys
    }

    /// Get the list of payment keys associated with this role.
    #[must_use]
    pub fn payment_keys(&self) -> &Vec<PointData<ShelleyAddress>> {
        &self.payment_keys
    }

    /// Get the list of extended data associated with this role.
    #[must_use]
    pub fn extended_data(&self) -> &Vec<PointData<HashMap<u8, Vec<u8>>>> {
        &self.extended_data
    }

    /// Get the signing key data associated with this role and the given key rotation.
    /// The first signing key is at rotation 0, the second at rotation 1, and so on.
    /// Returns `None` if the given key rotation does not exist.
    #[must_use]
    pub fn signing_key_from_rotation(
        &self,
        rotation: &KeyRotation,
    ) -> Option<&CertOrPk> {
        rotation.get_key(&self.signing_keys).map(PointData::data)
    }

    /// Get the encryption key data associated with this role and the given key rotation.
    /// The first encryption key is at rotation 0, the second at rotation 1, and so on.
    /// Returns `None` if the given key rotation does not exist.
    #[must_use]
    pub fn encryption_key_from_rotation(
        &self,
        rotation: &KeyRotation,
    ) -> Option<&CertOrPk> {
        rotation.get_key(&self.encryption_keys).map(PointData::data)
    }
}
