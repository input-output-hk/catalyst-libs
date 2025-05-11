//! A validation signature wrapper.

use anyhow::{anyhow, Error};
use ed25519_dalek::Signature;

/// A validation signature.
///
/// The signature must be at least 1 byte and at most 64 bytes long.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidationSignature(Vec<u8>);

impl TryFrom<Vec<u8>> for ValidationSignature {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > 64 {
            return Err(anyhow!("Invalid length ({}), 1..=64 expected", value.len()));
        }

        Ok(Self(value))
    }
}

impl TryInto<Signature> for ValidationSignature {
    type Error = Error;

    fn try_into(self) -> Result<Signature, Self::Error> {
        let sig_bytes: [u8; 64] = self
            .0
            .try_into()
            .map_err(|_| anyhow!("Invalid Ed25519 signature length, expect 64"))?;
        Ok(Signature::from_bytes(&sig_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_length() {
        let error = ValidationSignature::try_from(Vec::new()).unwrap_err();
        assert!(format!("{error}").starts_with("Invalid length"));

        let error = ValidationSignature::try_from(vec![0; 65]).unwrap_err();
        assert!(format!("{error}").starts_with("Invalid length"));
    }
}
