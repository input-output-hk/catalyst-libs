//! COSE Signature Protected Header `kid` URI Authority.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use thiserror::Error;

/// Errors that can occur when parsing an `Authority` from a string.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Error)]
pub enum AuthorityError {
    /// The input string does not match any known authority.
    #[error("Unknown Authority: {input}")]
    UnknownAuthority {
        /// The input string.
        input: String,
    },
}

/// URI Authority
#[derive(Debug, Clone)]
pub enum Authority {
    /// Cardano Blockchain
    Cardano,
    /// Midnight Blockchain
    Midnight,
}

impl FromStr for Authority {
    type Err = AuthorityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cardano" => Ok(Authority::Cardano),
            "midnight" => Ok(Authority::Midnight),
            _ => {
                Err(AuthorityError::UnknownAuthority {
                    input: s.to_string(),
                })
            },
        }
    }
}

impl Display for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let authority = match self {
            Self::Cardano => "cardano",
            Self::Midnight => "midnight",
        };
        write!(f, "{authority}")
    }
}
