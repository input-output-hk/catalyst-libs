//! Cryptographic Algorithm in COSE SIGN protected header.

use std::fmt::{Display, Formatter};

use strum::VariantArray;

/// Cryptography Algorithm.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize, VariantArray)]
pub enum Algorithm {
    /// `EdDSA`
    EdDSA,
}

impl Default for Algorithm {
    fn default() -> Self {
        Self::EdDSA
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::EdDSA => write!(f, "EdDSA"),
        }
    }
}

impl From<Algorithm> for coset::iana::Algorithm {
    fn from(_: Algorithm) -> Self {
        coset::iana::Algorithm::EdDSA
    }
}

impl TryFrom<coset::iana::Algorithm> for Algorithm {
    type Error = anyhow::Error;

    fn try_from(value: coset::iana::Algorithm) -> Result<Self, Self::Error> {
        match value {
            coset::iana::Algorithm::EdDSA => Ok(Self::EdDSA),
            _ => {
                anyhow::bail!(
                    "Unsupported algorithm: {value:?}, Supported only: {:?}",
                    Algorithm::VARIANTS
                )
            },
        }
    }
}
