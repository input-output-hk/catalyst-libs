//! Elgamal objects decoding implementation

use anyhow::anyhow;

use super::{Ciphertext, GroupElement, PublicKey, Scalar, SecretKey};

impl PublicKey {
    /// `PublicKey` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE;

    /// Convert this `PublicKey` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }

    /// Attempt to construct a `PublicKey` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode group element field.
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        GroupElement::from_bytes(bytes).map(Self)
    }
}

impl SecretKey {
    /// `SecretKey` bytes size
    pub const BYTES_SIZE: usize = Scalar::BYTES_SIZE;

    /// Convert this `SecretKey` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }

    /// Attempt to construct a `SecretKey` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode scalar field.
    pub fn from_bytes(bytes: [u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Scalar::from_bytes(bytes).map(Self)
    }
}

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
    fn keys_to_bytes_from_bytes_test(s1: SecretKey) {
        let bytes = s1.to_bytes();
        let s2 = SecretKey::from_bytes(bytes).unwrap();
        assert_eq!(s1, s2);

        let p1 = s1.public_key();
        let bytes = p1.to_bytes();
        let p2 = PublicKey::from_bytes(&bytes).unwrap();
        assert_eq!(p1, p2);
    }

    #[proptest]
    fn ciphertext_to_bytes_from_bytes_test(c1: Ciphertext) {
        let bytes = c1.to_bytes();
        let c2 = Ciphertext::from_bytes(&bytes).unwrap();
        assert_eq!(c1, c2);
    }
}
