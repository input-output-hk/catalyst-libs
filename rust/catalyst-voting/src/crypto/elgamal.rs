//! Implementation of the lifted ``ElGamal`` crypto system, and combine with `ChaCha`
//! stream cipher to produce a hybrid encryption scheme.

use std::ops::{Add, Deref, Mul};

use anyhow::anyhow;
use rand_core::CryptoRngCore;

use crate::crypto::group::{GroupElement, Scalar};

/// ``ElGamal`` secret key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(Scalar);

/// ``ElGamal`` public key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(GroupElement);

/// ``ElGamal`` ciphertext, encrypted message with the public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(GroupElement, GroupElement);

impl Deref for SecretKey {
    type Target = Scalar;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PublicKey {
    type Target = GroupElement;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SecretKey {
    /// Generate a random `SecretKey` value from the random number generator.
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        Self(Scalar::random(rng))
    }

    /// Generate a corresponding `PublicKey`.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        PublicKey(GroupElement::GENERATOR.mul(&self.0))
    }
}

impl Ciphertext {
    /// `Ciphertext` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE * 2;

    /// Generate a zero `Ciphertext`.
    /// The same as encrypt a `Scalar::zero()` message and `Scalar::zero()` randomness.
    pub fn zero() -> Self {
        Ciphertext(GroupElement::zero(), GroupElement::zero())
    }

    /// Get the first element of the `Ciphertext`.
    pub fn first(&self) -> &GroupElement {
        &self.0
    }

    /// Get the second element of the `Ciphertext`.
    pub fn second(&self) -> &GroupElement {
        &self.1
    }

    /// Convert this `Ciphertext` to its underlying sequence of bytes.
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; Self::BYTES_SIZE];
        res[0..32].copy_from_slice(&self.0.to_bytes());
        res[32..64].copy_from_slice(&self.1.to_bytes());
        res
    }

    /// Attempt to construct a `Scalar` from a compressed value byte representation.
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

/// Given a `message` represented as a `Scalar`, return a ciphertext using the
/// lifted ``ElGamal`` mechanism.
/// Returns a ciphertext of type `Ciphertext`.
pub fn encrypt(message: &Scalar, public_key: &PublicKey, randomness: &Scalar) -> Ciphertext {
    let e1 = GroupElement::GENERATOR.mul(randomness);
    let e2 = &GroupElement::GENERATOR.mul(message) + &public_key.0.mul(randomness);
    Ciphertext(e1, e2)
}

/// Decrypt ``ElGamal`` `Ciphertext`, returns the original message represented as a
/// `GroupElement`.
pub fn decrypt(cipher: &Ciphertext, secret_key: &SecretKey) -> GroupElement {
    &(&cipher.0 * &secret_key.0.negate()) + &cipher.1
}

impl Mul<&Scalar> for &Ciphertext {
    type Output = Ciphertext;

    fn mul(self, rhs: &Scalar) -> Self::Output {
        Ciphertext(&self.0 * rhs, &self.1 * rhs)
    }
}

impl Add<&Ciphertext> for &Ciphertext {
    type Output = Ciphertext;

    fn add(self, rhs: &Ciphertext) -> Self::Output {
        Ciphertext(&self.0 + &rhs.0, &self.1 + &rhs.1)
    }
}

#[cfg(test)]
mod tests {
    use proptest::{
        arbitrary::any,
        prelude::{Arbitrary, BoxedStrategy, Strategy},
    };
    use test_strategy::proptest;

    use super::*;

    impl Arbitrary for SecretKey {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<Scalar>().prop_map(SecretKey).boxed()
        }
    }

    impl Arbitrary for Ciphertext {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(GroupElement, GroupElement)>()
                .prop_map(|(g1, g2)| Ciphertext(g1, g2))
                .boxed()
        }
    }

    #[proptest]
    fn ciphertext_to_bytes_from_bytes_test(c1: Ciphertext) {
        let bytes = c1.to_bytes();
        let c2 = Ciphertext::from_bytes(&bytes).unwrap();
        assert_eq!(c1, c2);
    }

    #[proptest]
    fn ciphertext_add_test(e1: Scalar, e2: Scalar, e3: Scalar, e4: Scalar) {
        let g1 = GroupElement::GENERATOR.mul(&e1);
        let g2 = GroupElement::GENERATOR.mul(&e2);
        let c1 = Ciphertext(g1.clone(), g2.clone());

        let g3 = GroupElement::GENERATOR.mul(&e3);
        let g4 = GroupElement::GENERATOR.mul(&e4);
        let c2 = Ciphertext(g3.clone(), g4.clone());

        assert_eq!(&c1 + &c2, Ciphertext(&g1 + &g3, &g2 + &g4));
    }

    #[proptest]
    fn ciphertext_mul_test(e1: Scalar, e2: Scalar, e3: Scalar) {
        let g1 = GroupElement::GENERATOR.mul(&e1);
        let g2 = GroupElement::GENERATOR.mul(&e2);
        let c1 = Ciphertext(g1.clone(), g2.clone());

        assert_eq!(&c1 * &e3, Ciphertext(&g1 * &e3, &g2 * &e3));
    }

    #[proptest]
    fn elgamal_encryption_decryption_test(
        secret_key: SecretKey, message: Scalar, randomness: Scalar,
    ) {
        let public_key = secret_key.public_key();

        let cipher = encrypt(&message, &public_key, &randomness);
        let decrypted = decrypt(&cipher, &secret_key);

        assert_eq!(decrypted, GroupElement::GENERATOR.mul(&message));
    }
}
