//! Errors returned by this type

use displaydoc::Display;
use thiserror::Error;

use super::{key_rotation::KeyRotationError, role_index::RoleIdError};

/// Errors that can occur when parsing a `KidUri`
#[derive(Display, Error, Debug)]
pub enum CatalystIdError {
    /// {0} Invalid KID URI, err: {1}
    InvalidURI(String, fluent_uri::error::ParseError<String>),
    /// {0} Invalid Scheme, not a ID URI, {0}
    InvalidScheme(String),
    /// {0} Network not defined in URI
    NoDefinedNetwork(String),
    /// {0} Invalid Nonce
    InvalidNonce(String),
    /// {0} Path of URI is invalid
    InvalidPath(String),
    /// {0} Role 0 Key in path is invalid
    InvalidRole0Key(String),
    /// {0} Role 0 Key in path is not encoded correctly, err: {1}
    InvalidRole0KeyEncoding(String, base64_url::base64::DecodeError),
    /// {0} Role Index is invalid
    InvalidRole(String),
    /// {0} Role Index is not encoded correctly, err: {1}
    InvalidRoleId(String, RoleIdError),
    /// {0} Role Key Rotation is invalid
    InvalidRotation(String),
    /// {0} Role Key Rotation is not encoded correctly, err: {1}
    InvalidRotationValue(String, KeyRotationError),
    /// {0} Encryption key Identifier Fragment is not valid
    InvalidEncryptionKeyFragment(String),
    /// {0} Invalid Text encoding, err: {1}
    InvalidTextEncoding(String, std::string::FromUtf8Error),
}
