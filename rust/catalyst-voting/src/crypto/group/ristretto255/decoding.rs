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
        IScalar::from_canonical_bytes(bytes)
            .map(Scalar)
            .into_option()
            .ok_or(anyhow!("Cannot decode scalar."))
    }

    /// Convert this `Scalar` to its underlying sequence of bytes.
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
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.compress().to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn scalar_to_bytes_from_bytes_test(e1: Scalar) {
        let bytes = e1.to_bytes();
        let e2 = Scalar::from_bytes(bytes).unwrap();
        assert_eq!(e1, e2);
    }

    #[proptest]
    fn group_element_to_bytes_from_bytes_test(ge1: GroupElement) {
        let bytes = ge1.to_bytes();
        let ge2 = GroupElement::from_bytes(&bytes).unwrap();
        assert_eq!(ge1, ge2);
    }
}
