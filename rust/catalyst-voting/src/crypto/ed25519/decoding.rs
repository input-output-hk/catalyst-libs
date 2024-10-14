//! `Ed25519` objects decoding implementation

use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH};

use super::PublicKey;

impl PublicKey {
    /// `PublicKey` bytes size
    pub const BYTES_SIZE: usize = PUBLIC_KEY_LENGTH;

    /// Convert this `PublicKey` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }

    /// Attempt to construct a `PublicKey` from a byte representation.
    ///
    /// # Errors
    ///   - Cannot decode public key.
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(Self(VerifyingKey::from_bytes(bytes)?))
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::{super::PrivateKey, *};

    #[proptest]
    fn public_key_to_bytes_from_bytes_test(private_key: PrivateKey) {
        let public_key = private_key.public_key();
        let public_key_bytes = public_key.to_bytes();
        let public_key2 = PublicKey::from_bytes(&public_key_bytes).unwrap();
        assert_eq!(public_key, public_key2);
    }
}
