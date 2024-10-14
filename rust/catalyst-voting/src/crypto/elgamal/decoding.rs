//! Elgamal objects decoding implementation

use anyhow::anyhow;

use super::{Ciphertext, GroupElement};

impl Ciphertext {
    /// `Ciphertext` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE * 2;

    /// Convert this `Ciphertext` to its underlying sequence of bytes.
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; Self::BYTES_SIZE];
        res[0..32].copy_from_slice(&self.0.to_bytes());
        res[32..64].copy_from_slice(&self.1.to_bytes());
        res
    }

    /// Attempt to construct a `Ciphertext` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode group element field.
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(Self(
            GroupElement::from_bytes(bytes[0..32].try_into().unwrap())
                .map_err(|_| anyhow!("Cannot decode first group element field."))?,
            GroupElement::from_bytes(bytes[32..64].try_into().unwrap())
                .map_err(|_| anyhow!("Cannot decode second group element field."))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn ciphertext_to_bytes_from_bytes_test(c1: Ciphertext) {
        let bytes = c1.to_bytes();
        let c2 = Ciphertext::from_bytes(&bytes).unwrap();
        assert_eq!(c1, c2);
    }
}
