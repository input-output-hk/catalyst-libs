//! Errors returned by this type

use displaydoc::Display;
use thiserror::Error;

use super::{key_rotation::KeyRotationError, role_index::RoleIndexError};

/// Errors that can occur when parsing a `KidUri`
#[derive(Display, Error, Debug)]
pub enum IdUriError {
    /// Invalid KID URI
    InvalidURI(#[from] fluent_uri::error::ParseError<String>),
    /// Invalid Scheme, not a ID URI
    InvalidScheme,
    /// Network not defined in URI
    NoDefinedNetwork,
    /// Invalid Nonce
    InvalidNonce,
    /// Path of URI is invalid
    InvalidPath,
    /// Role 0 Key in path is invalid
    InvalidRole0Key,
    /// Role 0 Key in path is not encoded correctly
    InvalidRole0KeyEncoding(#[from] base64::DecodeError),
    /// Role Index is invalid
    InvalidRole,
    /// Role Index is not encoded correctly
    InvalidRoleIndex(#[from] RoleIndexError),
    /// Role Key Rotation is invalid
    InvalidRotation,
    /// Role Key Rotation is not encoded correctly
    InvalidRotationValue(#[from] KeyRotationError),
    /// Encryption key Identifier Fragment is not valid
    InvalidEncryptionKeyFragment,
    /// Invalid Text encoding
    InvalidTextEncoding(#[from] std::string::FromUtf8Error),
}
