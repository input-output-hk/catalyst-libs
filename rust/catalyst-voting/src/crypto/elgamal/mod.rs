//! Implementation of the lifted `ElGamal` crypto system, and combine with `ChaCha`
//! stream cipher to produce a hybrid encryption scheme.

mod decoding;

use std::ops::{Add, Mul};

use crate::crypto::group::{GroupElement, Scalar};

/// `ElGamal` ciphertext, encrypted message with the public key.
///
/// The CBOR CDDL schema:
/// ```cddl
/// elgamal-ristretto255-encrypted-choice = [
///     c1: elgamal-ristretto255-group-element
///     c2: elgamal-ristretto255-group-element
/// ]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct Ciphertext(GroupElement, GroupElement);

impl Ciphertext {
    /// Generate a zero `Ciphertext`.
    /// The same as encrypt a `Scalar::zero()` message and `Scalar::zero()` randomness.
    pub fn zero() -> Self {
        Ciphertext(GroupElement::zero(), GroupElement::zero())
    }

    /// Creates a `Ciphertext` instance from the given elements.
    pub fn from_elements(
        first: GroupElement,
        second: GroupElement,
    ) -> Self {
        Self(first, second)
    }

    /// Get the first element of the `Ciphertext`.
    pub fn first(&self) -> &GroupElement {
        &self.0
    }

    /// Get the second element of the `Ciphertext`.
    pub fn second(&self) -> &GroupElement {
        &self.1
    }
}

/// Generate `ElGamal` public key from the secret key value.
pub fn generate_public_key(secret_key: &Scalar) -> GroupElement {
    GroupElement::GENERATOR.mul(secret_key)
}

/// Given a `message` represented as a `Scalar`, return a ciphertext using the
/// lifted `ElGamal` mechanism.
/// Returns a ciphertext of type `Ciphertext`.
pub fn encrypt(
    message: &Scalar,
    public_key: &GroupElement,
    randomness: &Scalar,
) -> Ciphertext {
    let e1 = GroupElement::GENERATOR.mul(randomness);
    let e2 = &GroupElement::GENERATOR.mul(message) + &public_key.mul(randomness);
    Ciphertext(e1, e2)
}

/// Decrypt `ElGamal` `Ciphertext`, returns the original message represented as a
/// `GroupElement`.
pub fn decrypt(
    cipher: &Ciphertext,
    secret_key: &Scalar,
) -> GroupElement {
    &(&cipher.0 * &secret_key.negate()) + &cipher.1
}

impl Mul<&Scalar> for &Ciphertext {
    type Output = Ciphertext;

    fn mul(
        self,
        rhs: &Scalar,
    ) -> Self::Output {
        Ciphertext(&self.0 * rhs, &self.1 * rhs)
    }
}

impl Add<&Ciphertext> for &Ciphertext {
    type Output = Ciphertext;

    fn add(
        self,
        rhs: &Ciphertext,
    ) -> Self::Output {
        Ciphertext(&self.0 + &rhs.0, &self.1 + &rhs.1)
    }
}

#[cfg(test)]
mod arbitrary_impl {
    use proptest::{
        arbitrary::any,
        prelude::{Arbitrary, BoxedStrategy, Strategy},
    };

    use super::{Ciphertext, GroupElement};

    impl Arbitrary for Ciphertext {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(GroupElement, GroupElement)>()
                .prop_map(|(g1, g2)| Ciphertext(g1, g2))
                .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn ciphertext_add_test(
        e1: Scalar,
        e2: Scalar,
        e3: Scalar,
        e4: Scalar,
    ) {
        let g1 = GroupElement::GENERATOR.mul(&e1);
        let g2 = GroupElement::GENERATOR.mul(&e2);
        let c1 = Ciphertext(g1.clone(), g2.clone());

        let g3 = GroupElement::GENERATOR.mul(&e3);
        let g4 = GroupElement::GENERATOR.mul(&e4);
        let c2 = Ciphertext(g3.clone(), g4.clone());

        assert_eq!(&c1 + &c2, Ciphertext(&g1 + &g3, &g2 + &g4));
    }

    #[proptest]
    fn ciphertext_mul_test(
        e1: Scalar,
        e2: Scalar,
        e3: Scalar,
    ) {
        let g1 = GroupElement::GENERATOR.mul(&e1);
        let g2 = GroupElement::GENERATOR.mul(&e2);
        let c1 = Ciphertext(g1.clone(), g2.clone());

        assert_eq!(&c1 * &e3, Ciphertext(&g1 * &e3, &g2 * &e3));
    }

    #[proptest]
    fn elgamal_encryption_decryption_test(
        secret_key: Scalar,
        message: Scalar,
        randomness: Scalar,
    ) {
        let public_key = generate_public_key(&secret_key);

        let cipher = encrypt(&message, &public_key, &randomness);
        let decrypted = decrypt(&cipher, &secret_key);

        assert_eq!(decrypted, GroupElement::GENERATOR.mul(&message));
    }
}
