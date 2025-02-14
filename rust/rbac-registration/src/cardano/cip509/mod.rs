//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

pub use cip509::Cip509;
#[allow(clippy::module_name_repetitions)]
pub use rbac::{C509Cert, Cip509RbacMetadata, SimplePublicKeyType, X509DerCert};
pub use types::{
    CertKeyHash, KeyLocalRef, LocalRefInt, Payment, PaymentHistory, PointTxnIdx, RoleData,
    RoleNumber, TxInputHash, ValidationSignature,
};
pub use utils::Cip0134UriSet;

#[allow(clippy::module_inception)]
mod cip509;
mod decode_context;
mod rbac;
mod types;
mod utils;
mod validation;
mod x509_chunks;
