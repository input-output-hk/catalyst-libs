//! Role Based Access Control (RBAC) metadata for CIP509.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-role-registration-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-role-registration-metadata/CIP-XXXX/x509-roles.cddl>

pub mod role_data;

pub use certs::{C509Cert, C509CertInMetadatumReference, X509DerCert};
pub use metadata::{Cip509RbacMetadata, Cip509RbacMetadataInt};
pub use pub_key::SimplePublicKeyType;
pub use role_data::RoleData;
pub use role_number::RoleNumber;

mod certs;
mod metadata;
mod pub_key;
mod role_number;
mod tag;
