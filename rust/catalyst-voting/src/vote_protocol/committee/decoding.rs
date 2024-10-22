//! committee objects decoding implementation

use anyhow::anyhow;

use super::{ElectionPublicKey, ElectionSecretKey, GroupElement, Scalar};

impl ElectionSecretKey {
    /// `ElectionSecretKey` bytes size
    pub const BYTES_SIZE: usize = Scalar::BYTES_SIZE;

    /// Convert this `ElectionSecretKey` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }

    /// Attempt to construct a `ElectionSecretKey` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode election secret key.
    pub fn from_bytes(bytes: [u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(Self(Scalar::from_bytes(bytes).map_err(|_| {
            anyhow!("Cannot decode election secret key.")
        })?))
    }
}

impl ElectionPublicKey {
    /// `ElectionPublicKey` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE;

    /// Convert this `ElectionPublicKey` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }

    /// Attempt to construct a `ElectionPublicKey` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode election public key.
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(Self(
            GroupElement::from_bytes(bytes).map_err(|_| anyhow!("Cannot decode public key."))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn election_keys_to_bytes_from_bytes_test(sk1: ElectionSecretKey) {
        let bytes = sk1.to_bytes();
        let sk2 = ElectionSecretKey::from_bytes(bytes).unwrap();
        assert_eq!(sk1, sk2);

        let pk1 = sk1.public_key();
        let bytes = pk1.to_bytes();
        let pk2 = ElectionPublicKey::from_bytes(&bytes).unwrap();
        assert_eq!(pk1, pk2);
    }
}
