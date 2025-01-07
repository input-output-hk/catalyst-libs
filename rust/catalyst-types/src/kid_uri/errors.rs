//! Errors returned by this type

use thiserror::Error;

use super::{key_rotation::KeyRotationError, role_index::RoleIndexError};

/// Errors that can occur when parsing a `KidUri`
#[derive(Error, Debug)]
pub enum KidUriError {
    /// Invalid KID URI
    #[error("Invalid URI")]
    InvalidURI(#[from] fluent_uri::error::ParseError),
    /// Invalid Scheme, not a KID URI
    #[error("Invalid Scheme, not a KID URI")]
    InvalidScheme,
    /// Network not defined in URI
    #[error("No defined Network")]
    NoDefinedNetwork,
    /// Path of URI is invalid
    #[error("Invalid Path")]
    InvalidPath,
    /// Role 0 Key in path is invalid
    #[error("Invalid Role 0 Key")]
    InvalidRole0Key,
    /// Role 0 Key in path is not encoded correctly
    #[error("Invalid Role 0 Key Encoding")]
    InvalidRole0KeyEncoding(#[from] base64_url::base64::DecodeError),
    /// Role Index is invalid
    #[error("Invalid Role")]
    InvalidRole,
    /// Role Index is not encoded correctly
    #[error("Invalid Role Index")]
    InvalidRoleIndex(#[from] RoleIndexError),
    /// Role Key Rotation is invalid
    #[error("Invalid Rotation")]
    InvalidRotation,
    /// Role Key Rotation is not encoded correctly
    #[error("Invalid Rotation Value")]
    InvalidRotationValue(#[from] KeyRotationError),
    /// Encryption key Identifier Fragment is not valid
    #[error("Invalid Encryption Key Fragment")]
    InvalidEncryptionKeyFragment,
}
