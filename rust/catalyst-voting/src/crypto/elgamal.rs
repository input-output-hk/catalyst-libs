//! Implementation of the lifted ``ElGamal`` crypto system, and combine with `ChaCha`
//! stream cipher to produce a hybrid encryption scheme.

use std::ops::Mul;

use rand_core::CryptoRngCore;

use super::group::{GroupElement, Scalar};

/// ``ElGamal`` secret key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(Scalar);

/// ``ElGamal`` public key.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PublicKey(GroupElement);

/// ``ElGamal`` ciphertext, encrypted message with the public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(GroupElement, GroupElement);

impl SecretKey {
    /// Generate a random `SecretKey` value from the random number generator.
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        Self(Scalar::random(rng))
    }

    /// Generate a corresponding `PublicKey`.
    pub fn public_key(&self) -> PublicKey {
        PublicKey(GroupElement::GENERATOR.mul(&self.0))
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

#[cfg(test)]
mod tests {
    use proptest::{
        arbitrary::any,
        prelude::{Arbitrary, BoxedStrategy, Strategy},
        property_test,
    };

    use super::*;

    impl Arbitrary for SecretKey {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<Scalar>().prop_map(SecretKey).boxed()
        }
    }

    #[property_test]
    fn elgamal_encryption_decryption_test(
        secret_key: SecretKey, message: Scalar, randomness: Scalar,
    ) {
        let public_key = secret_key.public_key();

        let cipher = encrypt(&message, &public_key, &randomness);
        let decrypted = decrypt(&cipher, &secret_key);

        assert_ne!(decrypted, GroupElement::GENERATOR.mul(&message));
    }
}
