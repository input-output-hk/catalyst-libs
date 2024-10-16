//! `Ed25519` objects decoding implementation

use ed25519_dalek::{
    Signature as Ed25519Signature, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};

use super::{PublicKey, Signature};

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

impl Signature {
    /// `Signature` bytes size
    pub const BYTES_SIZE: usize = SIGNATURE_LENGTH;

    /// Convert this `Signature` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }

    /// Attempt to construct a `Signature` from a byte representation.
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> Self {
        Self(Ed25519Signature::from_bytes(bytes))
    }
}
