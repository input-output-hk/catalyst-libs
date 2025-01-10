//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

pub use cip509::Cip509;
pub use rbac::{
    role_data::{self, RoleData},
    C509Cert, RoleNumber, SimplePublicKeyType, X509DerCert,
};
pub use types::CertKeyHash;
pub use utils::Cip0134UriSet;

mod cip509;
mod rbac;
mod types;
mod utils;
mod validation;
mod x509_chunks;
