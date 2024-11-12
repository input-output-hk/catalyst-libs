//! ristretto255 objects decoding implementation

use anyhow::anyhow;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar as IScalar};

use super::{GroupElement, Scalar};

impl Scalar {
    /// `Scalar` bytes size
    pub const BYTES_SIZE: usize = 32;

    /// Attempt to construct a `Scalar` from a canonical byte representation.
    ///
    /// # Errors
    ///   - Cannot decode scalar.
    pub fn from_bytes(bytes: [u8; Self::BYTES_SIZE]) -> anyhow::Result<Scalar> {
        Into::<Option<_>>::into(IScalar::from_canonical_bytes(bytes))
            .map(Scalar)
            .ok_or(anyhow!("Cannot decode scalar."))
    }

    /// Convert this `Scalar` to its underlying sequence of bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }
}

impl GroupElement {
    /// `Scalar` bytes size
    pub const BYTES_SIZE: usize = 32;

    /// Attempt to construct a `Scalar` from a compressed value byte representation.
    ///
    /// # Errors
    ///   - Cannot decode group element.
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(GroupElement(
            CompressedRistretto::from_slice(bytes)?
                .decompress()
                .ok_or(anyhow!("Cannot decode group element."))?,
        ))
    }

    /// Convert this `GroupElement` to its underlying sequence of bytes.
    /// Always encode the compressed value.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.compress().to_bytes()
    }
}
