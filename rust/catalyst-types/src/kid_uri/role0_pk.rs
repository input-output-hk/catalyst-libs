//! COSE Signature Protected Header `kid` URI Role0 Public Key.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use thiserror::Error;

/// Errors that can occur when parsing a `Role0PublicKey` from a string.
#[derive(Debug, Error)]
pub enum Role0PublicKeyError {
    /// The input string does not start with the required "0x" prefix.
    #[error("Role0 Public Key hex string must start with '0x': {input}")]
    MissingPrefix {
        /// The input string.
        input: String,
    },
    /// The input string is not a valid hex-encoded string.
    #[error("Role0 Public Key is not a valid hex string: {source}")]
    InvalidHex {
        /// The underlying error from `hex`.
        #[from]
        source: hex::FromHexError,
    },
    /// The decoded key does not have the required length of 32 bytes.
    #[error("Role0 Public Key must have 32 bytes: {input}, len: {len}")]
    InvalidLength {
        /// The input string.
        input: String,
        /// The actual length of the input.
        len: usize,
    },
    /// Unexpected error during key conversion.
    #[error("Unable to read Role0 Public Key, this should never happen")]
    ConversionError,
}

/// Role0 Public Key.
#[derive(Debug, Clone)]
pub struct Role0PublicKey([u8; 32]);

impl FromStr for Role0PublicKey {
    type Err = Role0PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let role0_hex = s.strip_prefix("0x").ok_or_else(|| {
            Role0PublicKeyError::MissingPrefix {
                input: s.to_string(),
            }
        })?;

        let role0_key = hex::decode(role0_hex).map_err(Role0PublicKeyError::from)?;

        if role0_key.len() != 32 {
            return Err(Role0PublicKeyError::InvalidLength {
                input: role0_hex.to_string(),
                len: role0_key.len(),
            });
        }

        let role0: [u8; 32] = role0_key
            .try_into()
            .map_err(|_| Role0PublicKeyError::ConversionError)?;

        Ok(Role0PublicKey(role0))
    }
}

impl Display for Role0PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "0x{}", hex::encode(self.0))
    }
}
