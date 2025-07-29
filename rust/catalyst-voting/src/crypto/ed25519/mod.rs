//! `EdDSA` digital signature scheme over Curve25519.

mod decoding;

use ed25519_dalek::{
    ed25519::signature::Signer, Signature as Ed25519Signature, SigningKey, VerifyingKey,
};

use super::rng::default_rng;
use crate::crypto::rng::rand_core::CryptoRngCore;

/// `Ed25519` private key struct.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateKey(SigningKey);

impl PrivateKey {
    /// Randomly generate the `Ed25519` private key.
    pub fn random<R: CryptoRngCore>(rng: &mut R) -> Self {
        Self(SigningKey::generate(rng))
    }

    /// Randomly generate the `ElectionSecretKey` with the `crypto::default_rng`.
    pub fn random_with_default_rng() -> Self {
        Self::random(&mut default_rng())
    }

    /// Get associated `Ed25519` public key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }
}

/// `Ed25519` public key struct.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(VerifyingKey);

/// `Ed25519` signature struct.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(Ed25519Signature);

/// Sign a message using the `Ed25519` private key.
pub fn sign(sk: &PrivateKey, msg: &[u8]) -> Signature {
    Signature(sk.0.sign(msg))
}

/// Verify a `Ed25519` signature using the `Ed25519` public key.
#[must_use]
pub fn verify_signature(pk: &PublicKey, msg: &[u8], sig: &Signature) -> bool {
    pk.0.verify_strict(msg, &sig.0).is_ok()
}

#[cfg(test)]
mod arbitrary_impl {
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Strategy};

    use super::{PrivateKey, SigningKey};

    impl Arbitrary for PrivateKey {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<[u8; 32]>()
                .prop_map(|b| PrivateKey(SigningKey::from_bytes(&b)))
                .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn sign_test(private_key: PrivateKey, msg: Vec<u8>) {
        let public_key = private_key.public_key();
        let signature = sign(&private_key, &msg);
        assert!(verify_signature(&public_key, &msg, &signature));
    }
}
