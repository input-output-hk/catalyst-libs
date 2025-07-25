//! Helper functions to update the registration chain.

use std::collections::HashMap;

use c509_certificate::c509::C509;
use catalyst_types::catalyst_id::role_index::RoleId;
use ed25519_dalek::VerifyingKey;
use tracing::warn;
use x509_cert::certificate::Certificate as X509Certificate;

use crate::cardano::cip509::{
    C509Cert, CertKeyHash, CertOrPk, Cip509RbacMetadata, KeyLocalRef, LocalRefInt, PointData,
    PointTxnIdx, RoleData, RoleDataRecord, SimplePublicKeyType, X509DerCert,
};

/// Update x509 certificates in the registration chain.
pub(crate) fn update_x509_certs(
    x509_cert_map: &mut HashMap<usize, Vec<PointData<Option<X509Certificate>>>>,
    x509_certs: Vec<X509DerCert>,
    point_tx_idx: &PointTxnIdx,
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
pub(crate) fn update_c509_certs(
    c509_cert_map: &mut HashMap<usize, Vec<PointData<Option<C509>>>>,
    c509_certs: Vec<C509Cert>,
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
pub(crate) fn update_public_keys(
    pub_key_map: &mut HashMap<usize, Vec<PointData<Option<VerifyingKey>>>>,
    pub_keys: Vec<SimplePublicKeyType>,
    point_tx_idx: &PointTxnIdx,
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
pub(crate) fn revocations_list(
    revocation_list: Vec<CertKeyHash>,
    point_tx_idx: &PointTxnIdx,
) -> Vec<PointData<CertKeyHash>> {
    let mut revocations = Vec::new();
    for item in revocation_list {
        let point_data = PointData::new(point_tx_idx.clone(), item.clone());
        revocations.push(point_data);
    }
    revocations
}

/// Update the role data related fields in the registration chain.
pub(crate) fn update_role_data(
    registration: &Cip509RbacMetadata,
    role_data_history: &mut HashMap<RoleId, Vec<PointData<RoleData>>>,
    role_data_record: &mut HashMap<RoleId, RoleDataRecord>,
    point_tx_idx: &PointTxnIdx,
) {
    for (number, data) in registration.clone().role_data {
        // Update role data history, put the whole role data
        role_data_history
            .entry(number)
            .or_default()
            .push(PointData::new(point_tx_idx.clone(), data.clone()));

        // Update role data record
        let record = role_data_record
            .entry(number)
            .or_insert(RoleDataRecord::new());

        // Add signing key
        if let Some(signing_key) = data.signing_key() {
            update_signing_key(signing_key, record, point_tx_idx, registration);
        }

        // Add encryption key
        if let Some(encryption_key) = data.encryption_key() {
            update_encryption_key(encryption_key, record, point_tx_idx, registration);
        }

        // Add payment key
        if let Some(payment_key) = data.payment_key() {
            record.add_payment_key(PointData::new(point_tx_idx.clone(), payment_key.clone()));
        }

        // Add extended data
        record.add_extended_data(PointData::new(
            point_tx_idx.clone(),
            data.extended_data().clone(),
        ));
    }
}

/// Update signing key.
pub(crate) fn update_signing_key(
    signing_key: &KeyLocalRef,
    record: &mut RoleDataRecord,
    point_tx_idx: &PointTxnIdx,
    registration: &Cip509RbacMetadata,
) {
    let index = signing_key.key_offset;

    match signing_key.local_ref {
        LocalRefInt::X509Certs => {
            if let Some(cert) = registration.x509_certs.get(index) {
                match cert {
                    X509DerCert::Deleted => {
                        record.add_signing_key(CertOrPk::X509(None), point_tx_idx);
                    },
                    X509DerCert::X509Cert(c) => {
                        record
                            .add_signing_key(CertOrPk::X509(Some(c.clone().into())), point_tx_idx);
                    },
                    X509DerCert::Undefined => {},
                }
            }
        },
        LocalRefInt::C509Certs => {
            if let Some(cert) = registration.c509_certs.get(index) {
                match cert {
                    C509Cert::Deleted => {
                        record.add_signing_key(CertOrPk::C509(None), point_tx_idx);
                    },
                    C509Cert::C509Certificate(c) => {
                        record
                            .add_signing_key(CertOrPk::C509(Some(c.clone().into())), point_tx_idx);
                    },
                    C509Cert::Undefined | C509Cert::C509CertInMetadatumReference(_) => {},
                }
            }
        },
        LocalRefInt::PubKeys => {
            if let Some(key) = registration.pub_keys.get(index) {
                match key {
                    SimplePublicKeyType::Deleted => {
                        record.add_signing_key(CertOrPk::PublicKey(None), point_tx_idx);
                    },
                    SimplePublicKeyType::Ed25519(k) => {
                        record.add_signing_key(CertOrPk::PublicKey(Some(*k)), point_tx_idx);
                    },
                    SimplePublicKeyType::Undefined => {},
                }
            }
        },
    }
}

/// Update encryption key.
pub(crate) fn update_encryption_key(
    encryption_key: &KeyLocalRef,
    record: &mut RoleDataRecord,
    point_tx_idx: &PointTxnIdx,
    registration: &Cip509RbacMetadata,
) {
    let index = encryption_key.key_offset;

    match encryption_key.local_ref {
        LocalRefInt::X509Certs => {
            if let Some(cert) = registration.x509_certs.get(index) {
                match cert {
                    X509DerCert::Deleted => {
                        record.add_encryption_key(CertOrPk::X509(None), point_tx_idx);
                    },
                    X509DerCert::X509Cert(c) => {
                        record.add_encryption_key(
                            CertOrPk::X509(Some(c.clone().into())),
                            point_tx_idx,
                        );
                    },
                    X509DerCert::Undefined => {},
                }
            }
        },
        LocalRefInt::C509Certs => {
            if let Some(cert) = registration.c509_certs.get(index) {
                match cert {
                    C509Cert::Deleted => {
                        record.add_encryption_key(CertOrPk::C509(None), point_tx_idx);
                    },
                    C509Cert::C509Certificate(c) => {
                        record.add_encryption_key(
                            CertOrPk::C509(Some(c.clone().into())),
                            point_tx_idx,
                        );
                    },
                    C509Cert::Undefined | C509Cert::C509CertInMetadatumReference(_) => {},
                }
            }
        },
        LocalRefInt::PubKeys => {
            if let Some(key) = registration.pub_keys.get(index) {
                match key {
                    SimplePublicKeyType::Deleted => {
                        record.add_encryption_key(CertOrPk::PublicKey(None), point_tx_idx);
                    },
                    SimplePublicKeyType::Ed25519(k) => {
                        record.add_encryption_key(CertOrPk::PublicKey(Some(*k)), point_tx_idx);
                    },
                    SimplePublicKeyType::Undefined => {},
                }
            }
        },
    }
}
