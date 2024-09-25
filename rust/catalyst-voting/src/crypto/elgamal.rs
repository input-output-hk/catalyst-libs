//! Implementation of the lifted ``ElGamal`` cryptosystem, and combine with `ChaCha`
//! stream cipher to produce a hybrid encryption scheme.

use std::ops::{Add, Mul};

use rand_core::CryptoRngCore;

use super::group::ristretto255::{GroupElement, Scalar};

/// ``ElGamal`` secret key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(Scalar);

/// ``ElGamal`` public key.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PublicKey(GroupElement);

/// ``ElGamal`` ciphertext, encrypted message with the public key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(GroupElement, GroupElement);

/// Given a `message` represented as a `Scalar`, return a ciphertext using the
/// lifted ``ElGamal`` mechanism.
/// Returns a ciphertext of type `Ciphertext` and a used randomness.
pub fn encrypt<R: CryptoRngCore>(
    message: &Scalar, public_key: &PublicKey, rng: &mut R,
) -> (Ciphertext, Scalar) {
    let r = Scalar::random(rng);
    let e1 = GroupElement::GENERATOR.mul(&r);

    let a = GroupElement::GENERATOR.mul(message);
    let b = public_key.0.mul(&r);
    let e2 = a.add(&b);
    (Ciphertext(e1, e2), r)
}
